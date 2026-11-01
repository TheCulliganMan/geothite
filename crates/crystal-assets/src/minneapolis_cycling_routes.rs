//! Reposition the existing optional cycling cast on separate native streets.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::CompiledGamePack;
use crate::authored_town_geometry::kind;

const CITY: &str = "MnMinneapolis";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    schema_version: u8,
    map: String,
    native_module_sha256: String,
    routes: Vec<Route>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Route {
    id: String,
    name: String,
    before_start: [u16; 2],
    before_size: [u16; 2],
    start: [u16; 2],
    size: [u16; 2],
}

impl Route {
    fn rebind(&self, command: &str, args: &mut [String]) -> Result<bool> {
        if command != "object_event" || args.get(11) != Some(&self.id) {
            return Ok(false);
        }
        ensure!(
            args.len() == 13
                && args[0] == self.before_start[0].to_string()
                && args[1] == self.before_start[1].to_string()
                && args[3] == "SPRITEMOVEDATA_PATROL_CLOCKWISE"
                && args[4] == self.before_size[0].to_string()
                && args[5] == self.before_size[1].to_string(),
            "cycling event preimage differs: {}",
            self.id
        );
        args[0] = self.start[0].to_string();
        args[1] = self.start[1].to_string();
        args[4] = self.size[0].to_string();
        args[5] = self.size[1].to_string();
        Ok(true)
    }
}

fn rebind_raw(body: &mut Value, routes: &[Route]) -> Result<()> {
    let commands = body.as_array_mut().context("cycling raw event section")?;
    for route in routes {
        let mut count = 0;
        for row in &mut *commands {
            if row["command"] != "object_event" || row["args"][11] != route.id {
                continue;
            }
            let mut args: Vec<String> = serde_json::from_value(row["args"].clone())?;
            count += usize::from(route.rebind("object_event", &mut args)?);
            row["args"] = serde_json::to_value(args)?;
        }
        ensure!(count == 1, "one raw cycling event required: {}", route.id);
    }
    Ok(())
}

/// Moves all 18 existing cyclists onto distinct, collision-checked street loops.
/// Only their starts and native patrol extents change. Their sprites, optional
/// challenges, teams, dialogue and battle-return commands remain exact.
///
/// # Errors
/// Rejects stale city data, incomplete casts, overlapping routes, obstructed
/// streets, occupied arrivals, or inconsistent typed/raw event mirrors.
pub fn revise_minneapolis_cycling_routes(
    base: &CompiledGamePack,
    source: &str,
) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    crate::validate_compact_superior_geography(base)?;
    let source: Source = serde_json::from_str(source)?;
    let old = base.data.maps.get(CITY).context("installed Minneapolis")?;
    let canonical = serde_json::to_vec(&serde_json::to_value(old)?)?;
    ensure!(
        source.schema_version == 1
            && source.map == CITY
            && source.native_module_sha256 == format!("{:x}", Sha256::digest(canonical))
            && old.attributes.width == 40
            && old.attributes.height == 32,
        "cycling city preimage differs"
    );
    let expected: BTreeSet<_> = (1..=18)
        .map(|i| format!("MN_EXTRA_MPLS_CYCLE_{i:02}"))
        .collect();
    ensure!(
        source.routes.len() == 18
            && source
                .routes
                .iter()
                .map(|r| r.id.clone())
                .collect::<BTreeSet<_>>()
                == expected,
        "complete unique cycling cast required"
    );
    let mut protected: BTreeSet<_> = old
        .objects
        .iter()
        .filter(|o| !expected.contains(&o.script))
        .map(|o| [o.x, o.y])
        .chain(old.events.warps.iter().map(|w| [w.x, w.y]))
        .chain(old.events.coord_events.iter().map(|e| [e.x, e.y]))
        .chain(old.events.bg_events.iter().map(|e| [e.x, e.y]))
        .collect();
    protected.insert([40, 48]);
    let mut streets = BTreeSet::new();
    let mut city = old.clone();
    for route in &source.routes {
        let [x, y] = route.start;
        let [width, height] = route.size;
        ensure!(
            !route.name.trim().is_empty()
                && (1..=15).contains(&width)
                && (1..=15).contains(&height)
                && x > 0
                && y > 0
                && x.checked_add(width).is_some_and(|v| v < 79)
                && y.checked_add(height).is_some_and(|v| v < 63),
            "invalid native cycling extent: {}",
            route.id
        );
        for py in y..=y + height {
            for px in x..=x + width {
                if px != x && px != x + width && py != y && py != y + height {
                    continue;
                }
                let at = [px, py];
                ensure!(
                    matches!(
                        kind(base, old, at)?,
                        "FLOOR" | "BRIDGE_NS_DECK" | "BRIDGE_EW_DECK"
                    ) && !protected.contains(&at)
                        && streets.insert(at),
                    "cycling route overlaps a route, obstacle or arrival: {} at {px},{py}",
                    route.id
                );
            }
        }
        let matches: Vec<_> = city
            .objects
            .iter_mut()
            .filter(|o| o.script == route.id)
            .collect();
        ensure!(matches.len() == 1, "one installed cyclist required");
        let rider = matches.into_iter().next().context("cycling actor")?;
        ensure!(
            [rider.x, rider.y] == route.before_start
                && [rider.move_range_x, rider.move_range_y] == route.before_size
                && rider.spritemovedata == "SPRITEMOVEDATA_PATROL_CLOCKWISE"
                && rider.object_type == "OBJECTTYPE_SCRIPT"
                && rider.radius == 0
                && rider.object_identifier.as_deref() == Some(route.id.as_str()),
            "installed optional cyclist differs: {}",
            route.id
        );
        [rider.x, rider.y] = route.start;
        [rider.move_range_x, rider.move_range_y] = route.size;
        let mut count = 0;
        for row in &mut city.map_event_section_commands {
            count += usize::from(route.rebind(&row.command, &mut row.args)?);
        }
        ensure!(count == 1, "one typed cycling event required");
    }
    let label = old
        .attributes
        .map_events_label
        .as_ref()
        .context("city events label")?;
    if let Some(body) = city.scripts.get_mut(label) {
        rebind_raw(body, &source.routes)?;
    }
    let mut pack = base.clone();
    rebind_raw(
        pack.data
            .map_scripts
            .get_mut(label)
            .context("split city events")?,
        &source.routes,
    )?;
    pack.data
        .npcs
        .insert(CITY.into(), serde_json::to_value(&city.objects)?);
    pack.data.maps.insert(CITY.into(), city);
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    #[test]
    #[ignore = "requires CRYSTAL_CYCLING_PARENT and checked-in citywide route source"]
    fn cycling_publication_rejects_stale_overlap_and_blocked_routes() {
        let path = std::env::var_os("CRYSTAL_CYCLING_PARENT").expect("completed cycling parent");
        let pack = crate::read_verified_compiled_game_pack(PathBuf::from(path))
            .expect("verified cycling parent");
        let source = std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../planning/minnesota/revisions/minneapolis-citywide-cycling-v01/routes.json",
        ))
        .expect("runtime-loaded route fixture");
        let child = super::revise_minneapolis_cycling_routes(&pack, &source)
            .expect("distinct street routes");
        for (id, map) in &pack.data.maps {
            if id != super::CITY {
                assert_eq!(child.data.maps[id], *map, "unrelated map {id}");
            }
        }
        let before = &pack.data.maps[super::CITY];
        let after = &child.data.maps[super::CITY];
        assert_eq!(before.blocks, after.blocks);
        assert_eq!(before.events, after.events);
        assert_eq!(before.script_map_commands, after.script_map_commands);
        assert_eq!(child.runtime_files, pack.runtime_files);
        assert_eq!(child.audio_manifest, pack.audio_manifest);

        let original: serde_json::Value = serde_json::from_str(&source).expect("route JSON");
        for (field, value) in [
            ("native_module_sha256", serde_json::json!("stale")),
            ("schema_version", serde_json::json!(2)),
        ] {
            let mut invalid = original.clone();
            invalid[field] = value;
            assert!(super::revise_minneapolis_cycling_routes(&pack, &invalid.to_string()).is_err());
        }
        for point in [[0, 0], [63, 26], [48, 20]] {
            let mut invalid = original.clone();
            invalid["routes"][1]["start"] = serde_json::json!(point);
            assert!(super::revise_minneapolis_cycling_routes(&pack, &invalid.to_string()).is_err());
        }
    }
}
