//! Explicit hidden TM discoveries on existing Minnesota side paths.

use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::map::BackgroundEvent;
use serde::Deserialize;
use serde_json::json;

use crate::CompiledGamePack;
use crate::grand_marais_modpack::authoring;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HiddenCache {
    map: String,
    tile: [u16; 2],
    approach: [u16; 2],
    item: String,
}

pub(crate) fn install(pack: &mut CompiledGamePack, caches: &[HiddenCache]) -> Result<()> {
    ensure!(
        caches.len() == 20,
        "one authored cache per Minnesota route required"
    );
    let mut items = BTreeSet::new();
    let mut routes = BTreeSet::new();
    for cache in caches {
        let route = cache.map.get(..5).context("Minnesota route name")?;
        ensure!(
            route.starts_with("MnR") && routes.insert(route),
            "duplicate or invalid route cache"
        );
        ensure!(
            cache.item.starts_with("TM_") && items.insert(&cache.item),
            "unique TM discoveries required"
        );
        let item = pack
            .data
            .items
            .get(&cache.item)
            .context("installed cache TM")?;
        ensure!(
            item.tmhm_index
                .is_some_and(|index| (1..=50).contains(&index)),
            "cache must contain a native TM"
        );
        let mut map = pack
            .data
            .maps
            .get(&cache.map)
            .context("installed cache map")?
            .clone();
        let [x, y] = cache.tile;
        let [ax, ay] = cache.approach;
        ensure!(
            x.abs_diff(ax) + y.abs_diff(ay) == 1,
            "cache approach must be adjacent"
        );
        for tile in [cache.tile, cache.approach] {
            ensure!(
                crate::authored_town_geometry::kind(pack, &map, tile)? == "FLOOR",
                "cache and approach require native floor: {} {tile:?}",
                cache.map
            );
            ensure!(
                !map.events.bg_events.iter().any(|e| [e.x, e.y] == tile)
                    && !map.events.warps.iter().any(|e| [e.x, e.y] == tile)
                    && !map.events.coord_events.iter().any(|e| [e.x, e.y] == tile)
                    && !map.objects.iter().any(|o| [o.x, o.y] == tile),
                "cache overlaps an existing event: {} {tile:?}",
                cache.map
            );
        }
        let label = format!("MN_CACHE_{}", cache.item);
        let flag = format!("EVENT_{label}");
        ensure!(
            !pack.data.map_scripts.contains_key(&label),
            "occupied cache script"
        );
        crate::authored_city_expansion_events::flags(pack, std::slice::from_ref(&flag))?;
        map.scripts.insert(
            label.clone(),
            json!([authoring::command("hiddenitem", &[&cache.item, &flag])]),
        );
        map.events.bg_events.push(BackgroundEvent {
            x,
            y,
            event_type: "BGEVENT_ITEM".into(),
            script: label,
        });
        map.script_field_pickups =
            crate::parse_script_field_pickups(&map.id, &map.scripts, &map.objects)?;
        authoring::refresh_event_section(&mut map);
        crate::authored_town_publication::replace(pack, map)?;
    }
    Ok(())
}
