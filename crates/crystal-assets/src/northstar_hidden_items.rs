//! Deliberately placed discoveries using native hidden-item receipts and Itemfinder.

use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::map::BackgroundEvent;
use serde::Deserialize;
use serde_json::json;

use crate::authored_town_geometry::{kind, reachable};
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, MapModule};

const MANIFEST: &str = "northstar-hidden-discoveries-v1";
const PARENT: &str = "cd81b377a9d08285dddba8568547e573827860f1ea41630e2301be4750296313";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Discoveries {
    schema_version: u8,
    id: String,
    parent_content_hash: String,
    caches: Vec<Cache>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Cache {
    id: String,
    map: String,
    tile: [u16; 2],
    approach: [u16; 2],
    entry: [u16; 2],
    item: String,
    landmark: String,
    access: Access,
    /// Collision to the left, right, above and below the discovery.
    neighbors: [String; 4],
}

#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
enum Access {
    Walk,
    WaterDetour,
}

pub(crate) fn install(pack: &mut CompiledGamePack, source: &str) -> Result<()> {
    let discoveries: Discoveries = serde_json::from_str(source).context("hidden discoveries")?;
    ensure!(
        discoveries.schema_version == 1
            && discoveries.id == MANIFEST
            && discoveries.parent_content_hash == PARENT,
        "unsupported hidden discovery revision"
    );
    ensure!(
        discoveries.caches.len() == 40,
        "expected 40 authored discoveries"
    );
    ensure!(
        !pack.report.manifests.iter().any(|id| id == MANIFEST),
        "hidden discoveries are already installed"
    );
    let mut ids = BTreeSet::new();
    let mut maps = BTreeSet::new();
    for cache in &discoveries.caches {
        ensure!(
            !cache.id.is_empty()
                && cache.id.len() <= 32
                && cache
                    .id
                    .bytes()
                    .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == b'_')
                && ids.insert(&cache.id),
            "invalid or duplicate discovery ID: {}",
            cache.id
        );
        ensure!(
            cache.map.starts_with("Mn") && maps.insert(&cache.map),
            "discoveries require distinct Minnesota maps"
        );
        ensure!(
            matches!(
                cache.item.as_str(),
                "POTION"
                    | "ANTIDOTE"
                    | "PARLYZ_HEAL"
                    | "POKE_BALL"
                    | "GREAT_BALL"
                    | "SUPER_POTION"
                    | "FULL_HEAL"
                    | "REPEL"
                    | "SUPER_REPEL"
                    | "ETHER"
                    | "ELIXER"
                    | "NUGGET"
                    | "PP_UP"
                    | "RARE_CANDY"
                    | "ULTRA_BALL"
                    | "HYPER_POTION"
                    | "ESCAPE_ROPE"
                    | "REVIVE"
            ) && pack.data.items.contains_key(&cache.item),
            "hidden reward must be an installed ordinary item: {}",
            cache.item
        );
        let mut map = pack
            .data
            .maps
            .get(&cache.map)
            .context("discovery map")?
            .clone();
        validate_placement(pack, &map, cache)
            .with_context(|| format!("hidden discovery {}", cache.id))?;
        let label = format!("MN_HIDDEN_{}", cache.id);
        let flag = format!("EVENT_{label}");
        ensure!(
            !pack.data.map_scripts.contains_key(&label),
            "occupied hidden-item script: {label}"
        );
        crate::authored_city_expansion_events::flags(pack, std::slice::from_ref(&flag))?;
        map.scripts.insert(
            label.clone(),
            json!([authoring::command("hiddenitem", &[&cache.item, &flag])]),
        );
        map.events.bg_events.push(BackgroundEvent {
            x: cache.tile[0],
            y: cache.tile[1],
            event_type: "BGEVENT_ITEM".into(),
            script: label.clone(),
        });
        map.script_field_pickups =
            crate::parse_script_field_pickups(&map.id, &map.scripts, &map.objects)?;
        ensure!(
            map.script_field_pickups
                .iter()
                .any(|pickup| pickup.source_script == label
                    && pickup.command == "hiddenitem"
                    && pickup.item_id.as_deref() == Some(cache.item.as_str())
                    && pickup.event_flag.as_deref() == Some(flag.as_str())
                    && pickup.quantity == 1),
            "native pickup and Itemfinder metadata missing for {label}"
        );
        authoring::refresh_event_section(&mut map);
        crate::authored_town_publication::replace(pack, map)?;
    }
    for route in 1..=20 {
        ensure!(
            maps.contains(&format!("MnR{route:02}C00R00")),
            "missing Route {route} discovery"
        );
    }
    pack.report.manifests.push(MANIFEST.into());
    Ok(())
}

fn validate_placement(pack: &CompiledGamePack, map: &MapModule, cache: &Cache) -> Result<()> {
    let [x, y] = cache.tile;
    let [ax, ay] = cache.approach;
    ensure!(
        u32::from(x.abs_diff(ax)) + u32::from(y.abs_diff(ay)) == 1,
        "approach must be adjacent"
    );
    ensure!(
        !cache.landmark.trim().is_empty(),
        "discovery needs a landmark description"
    );
    for tile in [cache.tile, cache.approach] {
        ensure!(
            kind(pack, map, tile)? == "FLOOR",
            "discovery and approach need floor"
        );
        ensure!(
            !map.events
                .bg_events
                .iter()
                .any(|event| [event.x, event.y] == tile)
                && !map
                    .events
                    .warps
                    .iter()
                    .any(|event| [event.x, event.y] == tile)
                && !map
                    .events
                    .coord_events
                    .iter()
                    .any(|event| [event.x, event.y] == tile)
                && !map
                    .objects
                    .iter()
                    .any(|object| [object.x, object.y] == tile),
            "discovery overlaps an existing event or actor"
        );
    }
    let adjacent = [
        [x.checked_sub(1).context("left map edge")?, y],
        [x.checked_add(1).context("right map edge")?, y],
        [x, y.checked_sub(1).context("top map edge")?],
        [x, y.checked_add(1).context("bottom map edge")?],
    ];
    for (tile, expected) in adjacent.into_iter().zip(&cache.neighbors) {
        ensure!(
            kind(pack, map, tile)? == expected,
            "discovery landmark collision changed"
        );
    }
    let [ex, ey] = cache.entry;
    ensure!(
        kind(pack, map, cache.entry)? == "FLOOR",
        "entry needs native floor"
    );
    ensure!(
        map.events
            .warps
            .iter()
            .any(|warp| u32::from(ex.abs_diff(warp.x)) + u32::from(ey.abs_diff(warp.y)) <= 1)
            || map.attributes.connections.iter().any(|connection| {
                match connection.direction.as_str() {
                    "west" => ex == 0,
                    "east" => ex + 1 == map.attributes.width * 2,
                    "north" => ey == 0,
                    "south" => ey + 1 == map.attributes.height * 2,
                    _ => false,
                }
            }),
        "entry must adjoin an installed warp or connected edge"
    );
    // This proves local collision access from an authored entry, not earned
    // campaign access. It never creates saves, moves a player or runs scripts.
    let walking = reachable(pack, map, cache.entry, false)?;
    match cache.access {
        Access::Walk => ensure!(
            walking.contains(&cache.approach),
            "walking detour is disconnected"
        ),
        Access::WaterDetour => {
            ensure!(
                !walking.contains(&cache.approach),
                "water detour has a dry bypass from its entry"
            );
            ensure!(
                reachable(pack, map, cache.entry, true)?.contains(&cache.approach),
                "water detour is disconnected"
            );
        }
    }
    Ok(())
}
