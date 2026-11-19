//! Fit low shoreline stones and recess outdoor cave portals into their cliff faces.
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;

use super::{ArtIds, Room, append_metatiles, cliff_faces, paint_room, publish_room, source};
use crate::{CompiledGamePack, MapModule};

#[path = "shore_cave_links.rs"]
mod shore_cave_links;
#[path = "shore_rock_ink.rs"]
mod shore_rock_ink;

const MANIFEST: &str = "northstar-rocky-shores-v1";
const SECTIONS: [&str; 5] = [
    "routes",
    "places",
    "caves",
    "route-landings",
    "places-landings",
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Relocation {
    map: String,
    index: u16,
    from: [u16; 2],
    to: [u16; 2],
}

fn relocate(map: &mut MapModule, change: &Relocation) -> Result<()> {
    let [x, y] = change.to;
    let events = &mut map.events;
    ensure!(
        x < map.attributes.width * 2
            && y < map.attributes.height * 2
            && events.warps.iter().all(|w| [w.x, w.y] != change.to)
            && events.bg_events.iter().all(|e| [e.x, e.y] != change.to)
            && events.coord_events.iter().all(|e| [e.x, e.y] != change.to)
            && map.objects.iter().all(|o| [o.x, o.y] != change.to),
        "obstructed cave relocation on {} to [{x},{y}]",
        map.id
    );
    let i = usize::from(
        change
            .index
            .checked_sub(1)
            .context("expected one-based warp index")?,
    );
    let warp = events.warps.get_mut(i).context("missing outdoor warp")?;
    let row = map
        .map_event_section_commands
        .iter_mut()
        .filter(|r| r.command == "warp_event")
        .nth(i)
        .context("missing native warp record")?;
    let [sx, sy, target, exit] = row.args.as_mut_slice() else {
        anyhow::bail!("malformed native warp record")
    };
    ensure!(
        warp.index == change.index
            && [warp.x, warp.y] == change.from
            && sx.parse::<u16>()? == warp.x
            && sy.parse::<u16>()? == warp.y
            && *target == warp.target_map_constant
            && exit.parse::<i16>()? == warp.target_warp_id,
        "unexpected cave record on {} at {:?}",
        map.id,
        change.from
    );
    warp.x = x;
    warp.y = y;
    *sx = x.to_string();
    *sy = y.to_string();
    Ok(())
}

/// Adds the authored native rock metatiles and relocates cave entrances with their return tiles.
///
/// # Errors
/// Rejects a stale parent, occupied art or warp destination, changed shoreline permissions,
/// overlapping placements, broken cave links, or inaccessible public shore.
pub fn revise_rocky_shores(base: &CompiledGamePack, dir: &Path) -> Result<CompiledGamePack> {
    ensure!(
        base.report
            .manifests
            .last()
            .is_some_and(|m| m == cliff_faces::MANIFEST),
        "expected the cliff-faces review"
    );
    let mut pack = base.clone();
    let ink = shore_rock_ink::install(&mut pack, dir)?;
    let mut ids = ArtIds::new();
    let mut rooms = BTreeMap::<String, Room>::new();
    let mut protected = BTreeMap::<String, Vec<[u16; 2]>>::new();
    for section in SECTIONS {
        append_metatiles(
            &mut pack,
            &mut ids,
            source(dir, &format!("{section}-art.json"))?,
            |bank, g| ink.get(bank).is_some_and(|glyphs| glyphs.contains(&g)),
        )?;
        for mut room in source::<Vec<Room>>(dir, &format!("{section}.json"))? {
            let old = base
                .data
                .maps
                .get(&room.map)
                .context("missing rocky exterior")?;
            ensure!(
                room.map.starts_with("Mn")
                    && matches!(
                        old.attributes.environment.as_deref(),
                        Some("TOWN" | "ROUTE")
                    )
                    && room.tileset == old.attributes.tileset_name,
                "invalid outdoor map {}",
                room.map
            );
            if section != "caves" {
                protected
                    .entry(room.map.clone())
                    .or_default()
                    .extend(room.placements.iter().map(|p| p.at));
            }
            if let Some(existing) = rooms.get_mut(&room.map) {
                existing.placements.append(&mut room.placements);
            } else {
                rooms.insert(room.map.clone(), room);
            }
        }
    }
    let mut moves = BTreeMap::<String, Vec<Relocation>>::new();
    for entry in source::<Vec<Relocation>>(dir, "cave-warps.json")? {
        moves.entry(entry.map.clone()).or_default().push(entry);
    }
    for (name, room) in rooms {
        let old = &base.data.maps[&name];
        let mut map = paint_room(&pack, base, room, &ids)?;
        let bank = &map.attributes.tileset_name;
        for [x, y] in protected.remove(&name).unwrap_or_default() {
            let i = usize::from(y) * usize::from(map.attributes.width) + usize::from(x);
            let was = &base.data.tilesets[bank].collision[&format!("{:02x}", old.blocks[i])];
            let now = &pack.data.tilesets[bank].collision[&format!("{:02x}", map.blocks[i])];
            ensure!(
                was == now,
                "shore rock changes movement on {name} at {x},{y}"
            );
        }
        if let Some(relocations) = moves.remove(&name) {
            let mut seen = BTreeSet::new();
            for change in relocations {
                ensure!(
                    seen.insert(change.index),
                    "duplicate cave relocation on {name}"
                );
                relocate(&mut map, &change)?;
            }
        }
        publish_room(&mut pack, base, map)?;
    }
    ensure!(
        moves.is_empty(),
        "cave warp relocation has no accompanying map edit"
    );
    let root = dir.parent().context("missing revisions directory")?;
    cliff_faces::verify_installed_paths(&pack, &root.join("cliff-faces-v01"))?;
    for shoreline in [
        "route-shore-checks.json",
        "route-landing-edges.json",
        "places-landing-edges.json",
    ] {
        cliff_faces::verify_shores(&pack, dir, shoreline)?;
    }
    shore_cave_links::verify(base, &pack)?;
    for walk in source::<Vec<cliff_faces::WalkCheck>>(dir, "mouth-walks.json")? {
        walk.verify(&pack)?;
    }
    pack.report.manifests.push(MANIFEST.to_owned());
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
