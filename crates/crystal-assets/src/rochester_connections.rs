//! Native two-way walking seams from Winona through Driftless to Rochester.
use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use crystal_core::map::MapConnection;
use crystal_core::world::map::{OverworldMapData, TilePosition};
use crystal_core::world::session::connection_for_tile_checked;
use serde::Deserialize;

use crate::authored_checkpoint::{Checkpoint, install};
use crate::authored_town_geometry::{kind, reachable};
use crate::grand_marais_modpack::authoring;
use crate::{CompiledGamePack, rochester_publication};

#[derive(Deserialize)]
struct Source {
    manifest: String,
    links: Vec<Link>,
}
#[derive(Deserialize)]
struct Link {
    town: String,
    town_direction: String,
    route_direction: String,
    offset: i32,
    patch: [u16; 2],
    town_edge: [[u16; 2]; 2],
    route_edge: [[u16; 2]; 2],
}

/// Opens the actual two-tile R01 mouths and installs Winona's starter checkpoint.
///
/// Travel is reciprocal. Leaving Winona requires a real received partner;
/// returning is free and cannot strand a player. Rochester's western R02 mouth
/// remains closed. No reward, encounter table or route terrain is regenerated.
///
/// # Errors
/// Returns an error for stale/duplicate seams, unsafe mouths, missing native
/// constants, bypasses, mismatched offsets, parsing or runtime verification.
pub fn connect_winona_rochester(base: &CompiledGamePack, source: &str) -> Result<CompiledGamePack> {
    crate::verify_compiled_game_pack_for_runtime(base)?;
    let source: Source = serde_json::from_str(source).context("R01 town seams")?;
    ensure!(
        source.manifest == "minnesota-winona-rochester-links-v1"
            && source.links.len() == 2
            && !base.report.manifests.contains(&source.manifest),
        "R01 links duplicated or changed"
    );
    let mut maps = ["MnWinona", "MnRochester", "MnR01C00R00"]
        .into_iter()
        .map(|name| {
            Ok((
                name.to_owned(),
                base.data
                    .maps
                    .get(name)
                    .with_context(|| format!("installed {name}"))?
                    .clone(),
            ))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    for (link, (town, outward, reverse, patch, edge, route_x)) in source.links.iter().zip([
        ("MnWinona", "west", "east", [0, 9], 0, 63),
        ("MnRochester", "east", "west", [19, 9], 39, 0),
    ]) {
        ensure!(
            link.town == town
                && link.town_direction == outward
                && link.route_direction == reverse
                && link.offset == -3
                && link.patch == patch
                && link.town_edge == [[edge, 18], [edge, 19]]
                && link.route_edge == [[route_x, 24], [route_x, 25]],
            "fixed reciprocal R01 mouths and offsets required"
        );
        let map = maps.get_mut(town).context("town")?;
        ensure!(
            map.attributes.connections.is_empty(),
            "town already connected"
        );
        let index =
            usize::from(patch[1]) * usize::from(map.attributes.width) + usize::from(patch[0]);
        ensure!(map.blocks.get(index) == Some(&5), "stale closed town mouth");
        map.blocks[index] = 1;
        map.attributes.connections.push(MapConnection {
            direction: outward.into(),
            target_map: "MnR01C00R00".into(),
            offset: -3,
        });
        map.attributes.connection_flags = Some(outward.to_ascii_uppercase());
        let route = maps.get_mut("MnR01C00R00").context("route")?;
        ensure!(
            !route
                .attributes
                .connections
                .iter()
                .any(|c| c.direction == reverse),
            "route mouth already bound"
        );
        for at in link.route_edge {
            ensure!(
                kind(base, route, at)? == "FLOOR",
                "actual route seam must be floor"
            );
        }
        route.attributes.connections.push(MapConnection {
            direction: reverse.into(),
            target_map: town.into(),
            offset: 3,
        });
        route.attributes.connection_flags = Some(
            route
                .attributes
                .connections
                .iter()
                .map(|c| c.direction.to_ascii_uppercase())
                .collect::<Vec<_>>()
                .join(" | "),
        );
    }
    let mut winona = maps.remove("MnWinona").context("Winona")?;
    let ids = [
        "MN_W_TRAIL_WARDEN_NORTH".into(),
        "MN_W_TRAIL_WARDEN_SOUTH".into(),
    ];
    install(
        base,
        &mut winona,
        &Checkpoint {
            identifiers: &ids,
            bodies: [[2, 18], [2, 19]],
            inside: [[3, 18], [3, 19]],
            outside: [[1, 18], [1, 19]],
            root: "MnWTrailCheck",
            incoming_facing: "RIGHT",
            outward: "west",
            required_event: "EVENT_MN_STARTER_RECEIVED",
            badge_flags: &[],
            badge_lines: &["Take a partner".into(), "onto the trail.".into()],
            evidence_lines: &["Take a partner".into(), "onto the trail.".into()],
        },
    )?;
    for event in &mut winona.events.coord_events {
        if event.scene_id.is_empty() {
            event.scene_id = "-1".into();
        }
    }
    authoring::finish(base, &mut winona)?;
    winona.script_runtime_commands =
        crate::parse_script_runtime_commands(&winona.id, &winona.scripts)?;
    winona.script_object_commands =
        crate::parse_script_object_commands(&winona.id, &winona.scripts)?;
    winona.script_variable_commands =
        crate::parse_script_variable_commands(&winona.id, &winona.scripts)?;
    winona.script_movements =
        crate::parse_script_movements(&winona.id, &winona.scripts, &winona.script_object_commands)?;
    maps.insert(winona.id.clone(), winona);
    for name in ["MnWinona", "MnRochester"] {
        let m = &maps[name];
        let start = if name == "MnWinona" {
            [14, 18]
        } else {
            [32, 18]
        };
        let mut open = m.clone();
        open.objects.retain(|o| {
            !ids.iter()
                .any(|id| o.object_identifier.as_ref() == Some(id))
        });
        for at in reachable(base, &open, start, true)? {
            if at[0] == 0
                || at[0] == m.attributes.width * 2 - 1
                || at[1] == 0
                || at[1] == m.attributes.height * 2 - 1
            {
                let link = source
                    .links
                    .iter()
                    .find(|l| l.town == name)
                    .context("seam")?;
                ensure!(
                    link.town_edge.contains(&at),
                    "unbound town walking/Surf perimeter at {at:?}"
                );
            }
        }
    }
    let mut pack = base.clone();
    for (_, map) in maps {
        rochester_publication::replace(&mut pack, map)?;
    }
    for link in source.links {
        for (name, direction, edge, target, arrival) in [
            (
                link.town.as_str(),
                link.town_direction.as_str(),
                link.town_edge,
                "MnR01C00R00",
                link.route_edge,
            ),
            (
                "MnR01C00R00",
                link.route_direction.as_str(),
                link.route_edge,
                link.town.as_str(),
                link.town_edge,
            ),
        ] {
            let m = &pack.data.maps[name];
            let native = OverworldMapData::from_attributes(name, &m.attributes, m.blocks.clone());
            for (at, expected) in edge.into_iter().zip(arrival) {
                let crossing = TilePosition::new(
                    if direction == "west" {
                        -1
                    } else {
                        i16::try_from(at[0] + 1)?
                    },
                    i16::try_from(at[1])?,
                );
                let transition = connection_for_tile_checked(&native, crossing)?
                    .context("native edge transition")?;
                let resolved = pack.data.resolve_connection_transition(&transition)?;
                ensure!(
                    resolved.destination.map_name == target
                        && resolved.destination.tile
                            == TilePosition::new(
                                i16::try_from(expected[0])?,
                                i16::try_from(expected[1])?
                            ),
                    "native seam arrival differs"
                );
            }
        }
    }
    pack.report.manifests.push(source.manifest);
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
