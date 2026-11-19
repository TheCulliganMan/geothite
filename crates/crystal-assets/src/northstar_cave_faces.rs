//! Minnesota's exterior cave mouths and their supporting rock.
use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{Context, Result, ensure};

use super::rock_cut::finish_cave_revision;
use super::{ArtIds, Metatile, Room, append_metatiles, apply_room, metatile_path, source};
use crate::authored_town_geometry::kind;
use crate::outdoor_stair_landings::glyph_bytes;
use crate::{CompiledGamePack, MapModule};

fn exterior(room: &MapModule) -> bool {
    room.id.starts_with("Mn")
        && matches!(
            room.attributes.environment.as_deref(),
            Some("ROUTE" | "TOWN")
        )
}

// Look up the installed graphics, since the same stone has different tile numbers
// in Johto, Iron Range and Gunflint. Each walking square contains four 8px glyphs.
fn quadrant<'a>(
    pack: &'a CompiledGamePack,
    room: &MapModule,
    [x, y]: [u16; 2],
) -> Result<[&'a [u8]; 4]> {
    let attrs = &room.attributes;
    ensure!(
        x < attrs.width * 2 && y < attrs.height * 2,
        "outside {}",
        room.id
    );
    let bank = &attrs.tileset_name;
    let block = room.blocks[usize::from(y / 2) * usize::from(attrs.width) + usize::from(x / 2)];
    let layout = pack
        .runtime_files
        .get(&metatile_path(bank))
        .context("missing native cliff layout")?;
    let start = usize::from(block) * 16 + usize::from(y % 2) * 8 + usize::from(x % 2) * 2;
    let tiles = layout
        .get(start..start + 6)
        .context("incomplete native cliff block")?;
    Ok([
        glyph_bytes(pack, bank, tiles[0])?,
        glyph_bytes(pack, bank, tiles[1])?,
        glyph_bytes(pack, bank, tiles[4])?,
        glyph_bytes(pack, bank, tiles[5])?,
    ])
}

fn check_cliff_faces(pack: &CompiledGamePack) -> Result<()> {
    // The wall directly east of Mt. Mortar's western entrance in Crystal.
    let route = pack
        .data
        .maps
        .get("Route42")
        .context("missing Mt. Mortar exterior")?;
    let [stone, ..] = quadrant(pack, route, [11, 5])?;
    for room in pack.data.maps.values().filter(|r| exterior(r)) {
        for warp in &room.events.warps {
            let [x, y] = [warp.x, warp.y];
            if kind(pack, room, [x, y])? != "CAVE" {
                continue;
            }
            for dx in [-1, 1] {
                let nx = x.checked_add_signed(dx).context("missing cave shoulder")?;
                let [tl, tr, bl, br] = quadrant(pack, room, [nx, y])?;
                // Allow a timber jamb or a bevel next to an unbroken column of stone.
                ensure!(
                    kind(pack, room, [nx, y])? == "WALL"
                        && ((tl == stone && bl == stone) || (tr == stone && br == stone)),
                    "{} cave at ({x},{y}) lacks a full-height rock face at ({nx},{y})",
                    room.id
                );
            }
        }
    }
    Ok(())
}

// Field and Mothlight allocated the same timber doorway at different indices.
// Bind the source placement and its collision donor to the installed native door.
fn bind_mine_door(base: &CompiledGamePack, rooms: &mut [Room], art: &mut [Metatile]) -> Result<()> {
    let room = rooms
        .iter_mut()
        .find(|r| r.map == "MnR09C00R00")
        .context("missing mine exterior")?;
    let placement = room
        .placements
        .iter_mut()
        .find(|c| c.at == [20, 20])
        .context("missing mine doorway")?;
    let old = base.data.maps.get(&room.map).context("missing mine map")?;
    let [x, y] = placement.at.map(usize::from);
    let block = *old
        .blocks
        .get(y * usize::from(old.attributes.width) + x)
        .context("mine doorway outside map")?;
    let installed = u8::try_from(block)?;
    let tileset = &old.attributes.tileset_name;
    let layout = base
        .runtime_files
        .get(&metatile_path(tileset))
        .context("missing mine art")?;
    let offset = usize::from(installed) * 16;
    let original = [
        0x3c, 0x3c, 0x3c, 0x3c, 0xf3, 0xf4, 0xf5, 0x3c, 0xf6, 0xf7, 0xf8, 0x4c, 0xf9, 0xfa, 0xfb,
        0x4c,
    ];
    let collision = &base
        .data
        .tilesets
        .get(tileset)
        .context("missing mine tileset")?
        .collision;
    ensure!(
        matches!(installed, 0x88 | 0x89)
            && layout.get(offset..offset + 16) == Some(original.as_slice())
            && collision
                .get(&format!("{installed:02x}"))
                .is_some_and(|q| q == &["FLOOR", "FLOOR", "CAVE", "WALL"]),
        "unexpected original mine doorway at ({x},{y})"
    );
    let recipe = art
        .iter_mut()
        .find(|r| r.tileset == room.tileset && r.id == placement.to)
        .context("missing mine door artwork")?;
    ensure!(
        placement.from == 0x88 && recipe.collision_from == 0x88 && recipe.collision.is_none(),
        "unexpected mine door recipe"
    );
    placement.from = block;
    recipe.collision_from = installed;
    Ok(())
}

/// Fits native cliff fronts around Minnesota's cave entrances and checks their rock and paths.
///
/// # Errors
/// Rejects a different pack lineage, stale room or tile data, shallow cave shoulders,
/// or a broken entrance, landing, stair or route connection.
pub fn revise_crystal_cave_faces(base: &CompiledGamePack, dir: &Path) -> Result<CompiledGamePack> {
    ensure!(
        base.report
            .manifests
            .last()
            .is_some_and(|id| id == "northstar-rock-cut-north-face-v1"),
        "expected the preceding Rock Cut north face revision"
    );
    let mut pack = base.clone();
    let mut ids = ArtIds::new();
    let mut changed = BTreeSet::new();
    for group in ["rock-cut", "regional"] {
        let mut rooms: Vec<Room> = source(dir, &format!("{group}.json"))?;
        let mut recipes: Vec<Metatile> = source(dir, &format!("{group}-art.json"))?;
        if group == "regional" {
            bind_mine_door(base, &mut rooms, &mut recipes)?;
        }
        ensure!(
            recipes
                .iter()
                .all(|r| rooms.iter().any(|m| m.tileset == r.tileset)),
            "cave art references an unrelated bank"
        );
        append_metatiles(&mut pack, &mut ids, recipes, |_, _| false)?;
        for room in rooms {
            let original = base
                .data
                .maps
                .get(&room.map)
                .context("missing cave exterior")?;
            ensure!(
                exterior(original) && changed.insert(room.map.clone()),
                "unexpected or repeated cave exterior"
            );
            apply_room(&mut pack, base, room, &ids)?;
        }
    }
    check_cliff_faces(&pack)?;
    finish_cave_revision(base, pack, dir, "northstar-crystal-cave-faces-v1")
}
