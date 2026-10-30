//! Restore native outer corners where decorative boulders interrupted a wall.

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use crate::CompiledGamePack;

#[derive(Deserialize)]
pub(super) struct CornerBoulder {
    pub map: String,
    pub tileset: String,
    pub metatile: [usize; 2],
    pub corner: String,
    pub destination: Option<[usize; 2]>,
    pub destination_before: Option<String>,
}

/// Applies one guarded corner repair; the caller owns whole-module and cell guards.
pub(super) fn apply(pack: &mut CompiledGamePack, repair: &CornerBoulder) -> Result<()> {
    let [x, y] = repair.metatile;
    let (side, corner, expected) = match repair.corner.as_str() {
        "6c" => (0x68, 0x6c, ["WALL", "FLOOR", "WALL", "WALL"]),
        "6d" => (0x69, 0x6d, ["FLOOR", "WALL", "WALL", "WALL"]),
        _ => anyhow::bail!("boulder repair requires a native outer front corner"),
    };
    let map = pack
        .data
        .maps
        .get_mut(&repair.map)
        .context("corner exterior")?;
    let width = usize::from(map.attributes.width);
    ensure!(
        map.attributes.tileset_name == repair.tileset
            && x < width
            && y > 0
            && y < usize::from(map.attributes.height),
        "corner repair source is outside its native exterior"
    );
    let inside_x = if corner == 0x6c {
        x.checked_add(1)
    } else {
        x.checked_sub(1)
    }
    .filter(|inside| *inside < width)
    .context("corner has no interior front neighbor")?;
    let collision = &pack.data.tilesets[&repair.tileset].collision;
    ensure!(
        map.blocks[y * width + x] == 0x0a
            && map.blocks[(y - 1) * width + x] == side
            && map.blocks[y * width + inside_x] == 0x72
            && collision
                .get("0a")
                .is_some_and(|kinds| kinds.iter().all(|k| k == "WALL"))
            && collision
                .get(&repair.corner)
                .is_some_and(|kinds| kinds.iter().map(String::as_str).eq(expected)),
        "corner must continue its actual side wall using unchanged native art"
    );
    if let Some([dx, dy]) = repair.destination {
        ensure!(
            dx == inside_x && dy == y - 1,
            "corner boulder must move one tile inward and one north"
        );
        let before = repair
            .destination_before
            .as_deref()
            .context("corner boulder destination preimage")?;
        ensure!(
            map.blocks[dy * width + dx] == u16::from_str_radix(before, 16)?
                && collision.get(before).is_some_and(|kinds| kinds
                    .iter()
                    .all(|k| matches!(k.as_str(), "FLOOR" | "TALL_GRASS")))
                && !map
                    .objects
                    .iter()
                    .any(|o| usize::from(o.x) / 2 == dx && usize::from(o.y) / 2 == dy),
            "corner boulder destination must be unchanged unoccupied flat ground"
        );
        map.blocks[dy * width + dx] = 0x0a;
    } else {
        // These two end caps frame a two-walking-tile shelf. Moving either rock
        // inward would seal that complete corridor rather than decorate it.
        ensure!(
            repair.map == "MnR09C01R00"
                && y == 20
                && matches!((x, corner), (7, 0x6c) | (9, 0x6d))
                && repair.destination_before.is_none(),
            "only the reviewed narrow Iron Range shelf omits its clutter boulder"
        );
    }
    map.blocks[y * width + x] = corner;
    Ok(())
}
