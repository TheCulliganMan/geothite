//! Align the League walking entrance without shifting the connected world.
use anyhow::{Context, Result, ensure};

use crate::CompiledGamePack;

/// Aligns the League trail with both existing town clearance lanes.
///
/// Extends the trail's adjacent native floor by one metatile and restores the
/// reciprocal zero offsets required by the global outdoor connection cycle.
/// Existing actors, events, scripts, art, audio and other terrain are preserved.
/// Native clearance, walking and visual review remain required for acceptance.
///
/// # Errors
/// Rejects incompatible borders, occupied terrain, unexpected offsets, duplicate
/// publication, invalid packs or inconsistent resulting outdoor geography.
pub fn align_northstar_league_entrance(base: &CompiledGamePack) -> Result<CompiledGamePack> {
    const MANIFEST: &str = "northstar-canonical-league-entrance-v1";
    crate::verify_compiled_game_pack_for_runtime(base)?;
    ensure!(
        !base.report.manifests.iter().any(|name| name == MANIFEST),
        "League entrance alignment already published"
    );
    let mut pack = base.clone();
    install(&mut pack)?;
    crate::validate_compact_superior_geography(&pack)?;
    pack.report.manifests.push(MANIFEST.into());
    pack.identity = crate::derive_compiled_game_pack_identity_from_manifest(
        pack.format_version,
        &pack.data,
        &pack.audio_manifest,
        &pack.runtime_files,
        &pack.report,
    )?;
    crate::verify_compiled_game_pack_for_runtime(&pack)?;
    Ok(pack)
}

pub(crate) fn install(pack: &mut CompiledGamePack) -> Result<()> {
    for (name, y, xs) in [
        ("MnBoundaryWaters", 47, [32, 33]),
        ("MnR13C00R00", 0, [30, 31]),
    ] {
        let map = pack.data.maps.get(name).context("League walking border")?;
        ensure!(
            map.attributes.width == 32 && map.attributes.height == 24,
            "League border dimensions changed"
        );
        let collision = &pack.data.tilesets[&map.attributes.tileset_name].collision;
        for x in xs {
            let block = map.blocks[y / 2 * usize::from(map.attributes.width) + x / 2];
            ensure!(
                collision[&format!("{block:02x}")][y % 2 * 2 + x % 2] == "FLOOR",
                "League border floor changed: {name} ({x}, {y})"
            );
        }
    }
    // Camp's cleared exit is x32/33; the trail begins at x30/31. Shifting the
    // connection joins those floors but breaks the global coastal cycle. A
    // short native floor turnout joins them at the canonical world placement.
    let mut road = pack.data.maps["MnR13C00R00"].clone();
    let floor = road.blocks[15];
    let before = road.blocks[16];
    let collision = &pack.data.tilesets[&road.attributes.tileset_name].collision;
    ensure!(
        collision[&format!("{floor:02x}")]
            .iter()
            .all(|kind| kind == "FLOOR")
            && (before == floor
                || collision[&format!("{before:02x}")]
                    .iter()
                    .all(|kind| kind == "WALL")),
        "native League turnout preimage changed"
    );
    let occupied = |x: u16, y: u16| x / 2 == 16 && y / 2 == 0;
    ensure!(
        !road.objects.iter().any(|o| occupied(o.x, o.y))
            && !road.events.warps.iter().any(|w| occupied(w.x, w.y))
            && !road.events.coord_events.iter().any(|e| occupied(e.x, e.y))
            && !road.events.bg_events.iter().any(|e| occupied(e.x, e.y))
            && road.script_block_changes.is_empty(),
        "League turnout overlaps authored gameplay"
    );
    road.blocks[16] = floor;
    crate::authored_town_publication::replace(pack, road)?;
    for (name, direction, target, old_offset) in [
        ("MnBoundaryWaters", "south", "MnR13C00R00", 1),
        ("MnR13C00R00", "north", "MnBoundaryWaters", -1),
    ] {
        let mut map = pack.data.maps[name].clone();
        let mut matches = map
            .attributes
            .connections
            .iter_mut()
            .filter(|link| link.direction == direction && link.target_map == target);
        let link = matches.next().context("reciprocal League connection")?;
        ensure!(
            link.offset == 0 || link.offset == old_offset,
            "League connection offset changed"
        );
        link.offset = 0;
        ensure!(matches.next().is_none(), "duplicate League connection");
        crate::authored_town_publication::replace(pack, map)?;
    }
    Ok(())
}
