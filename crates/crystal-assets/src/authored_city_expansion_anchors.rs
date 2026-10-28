//! Exact relocation of installed events and reciprocal regional seams.

use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::world::map::{OverworldMapData, TilePosition};
use crystal_core::world::session::connection_for_tile_checked;

use crate::authored_city_expansion_design::{AnchorKind, Expansion};
use crate::authored_town_publication;
use crate::{CompiledGamePack, MapModule};

pub(super) fn relocate(map: &mut MapModule, plan: &Expansion) -> Result<()> {
    // These cities have no coordinate-bearing script instructions. Refuse a
    // future changed epoch instead of moving events under unmoved script data.
    ensure!(
        map.script_map_commands.is_empty()
            && map.script_block_changes.is_empty()
            && map
                .script_object_commands
                .iter()
                .all(|command| command.x.is_none() && command.y.is_none())
            && map.scenes.scenes.is_empty(),
        "city has coordinate-bearing story programs"
    );
    let shifted = |[x, y]: [u16; 2]| -> Result<[u16; 2]> {
        Ok([
            x.checked_add(plan.translation_core[0])
                .context("city x overflow")?,
            y.checked_add(plan.translation_core[1])
                .context("city y overflow")?,
        ])
    };
    let apply = |kind: AnchorKind, index: usize, x: &mut u16, y: &mut u16| -> Result<()> {
        let old = [*x, *y];
        let matching = plan
            .anchor_overrides
            .iter()
            .filter(|a| a.kind == kind && a.index == index)
            .collect::<Vec<_>>();
        ensure!(matching.len() <= 1, "duplicate anchor override");
        let after = if let Some(anchor) = matching.first() {
            ensure!(anchor.before == old, "stale city anchor index {index}");
            anchor.after
        } else {
            shifted(old)?
        };
        ensure!(
            after[0] < plan.dimensions_metatiles[0] * 2
                && after[1] < plan.dimensions_metatiles[1] * 2,
            "city anchor outside extent"
        );
        [*x, *y] = after;
        Ok(())
    };
    for (i, object) in map.objects.iter_mut().enumerate() {
        apply(AnchorKind::Object, i, &mut object.x, &mut object.y)?;
    }
    for (i, event) in map.events.bg_events.iter_mut().enumerate() {
        apply(AnchorKind::Background, i, &mut event.x, &mut event.y)?;
    }
    for (i, event) in map.events.coord_events.iter_mut().enumerate() {
        apply(AnchorKind::Coordinate, i, &mut event.x, &mut event.y)?;
    }
    for (i, warp) in map.events.warps.iter_mut().enumerate() {
        apply(AnchorKind::Warp, i, &mut warp.x, &mut warp.y)?;
    }
    for anchor in &plan.anchor_overrides {
        let len = match anchor.kind {
            AnchorKind::Object => map.objects.len(),
            AnchorKind::Background => map.events.bg_events.len(),
            AnchorKind::Coordinate => map.events.coord_events.len(),
            AnchorKind::Warp => map.events.warps.len(),
        };
        ensure!(
            anchor.index < len,
            "override references absent native anchor"
        );
    }
    Ok(())
}

pub(super) fn spawns(pack: &mut CompiledGamePack, plan: &Expansion) -> Result<()> {
    let mut used = BTreeSet::new();
    for spawn in pack
        .data
        .runtime_spawn_points
        .values_mut()
        .filter(|s| s.map_name == plan.city)
    {
        let old = [u16::try_from(spawn.tile_x)?, u16::try_from(spawn.tile_y)?];
        let matching = plan
            .spawn_overrides
            .iter()
            .filter(|s| s.identifier == spawn.identifier)
            .collect::<Vec<_>>();
        ensure!(matching.len() <= 1, "duplicate city spawn override");
        let after = if let Some(proposal) = matching.first() {
            ensure!(proposal.before == old, "stale city spawn preimage");
            used.insert(spawn.identifier);
            proposal.after
        } else {
            [
                old[0]
                    .checked_add(plan.translation_core[0])
                    .context("spawn x overflow")?,
                old[1]
                    .checked_add(plan.translation_core[1])
                    .context("spawn y overflow")?,
            ]
        };
        ensure!(
            after[0] < plan.dimensions_metatiles[0] * 2
                && after[1] < plan.dimensions_metatiles[1] * 2,
            "city spawn outside extent"
        );
        spawn.tile_x = i16::try_from(after[0])?;
        spawn.tile_y = i16::try_from(after[1])?;
        spawn.metatile_x = spawn.tile_x / 2;
        spawn.metatile_y = spawn.tile_y / 2;
        spawn.subtile_x = spawn.tile_x % 2;
        spawn.subtile_y = spawn.tile_y % 2;
    }
    ensure!(
        used.len() == plan.spawn_overrides.len(),
        "unknown city spawn override"
    );
    Ok(())
}

fn opposite(direction: &str) -> Result<&'static str> {
    Ok(match direction {
        "north" => "south",
        "south" => "north",
        "east" => "west",
        "west" => "east",
        _ => anyhow::bail!("invalid city connection direction"),
    })
}

pub(super) fn connections(
    pack: &mut CompiledGamePack,
    city: &mut MapModule,
    plan: &Expansion,
) -> Result<()> {
    connections_with_retained_water(pack, city, plan, &BTreeSet::new())
}

/// Retained river connections are independently paired by the water validator.
/// Their offsets and neighboring modules are not changed here.
pub(super) fn connections_with_retained_water(
    pack: &mut CompiledGamePack,
    city: &mut MapModule,
    plan: &Expansion,
    retained: &BTreeSet<(&str, &str)>,
) -> Result<()> {
    let declared = plan
        .connections
        .iter()
        .map(|link| (link.direction.as_str(), link.target.as_str()))
        .chain(retained.iter().copied())
        .collect::<BTreeSet<_>>();
    let actual = city
        .attributes
        .connections
        .iter()
        .map(|link| (link.direction.as_str(), link.target_map.as_str()))
        .collect::<BTreeSet<_>>();
    ensure!(
        actual == declared && actual.len() == city.attributes.connections.len(),
        "every native city connection needs a contract"
    );
    let mut used = BTreeSet::new();
    for link in &plan.connections {
        ensure!(
            used.insert(link.direction.clone()),
            "duplicate city connection direction"
        );
        let connection = city
            .attributes
            .connections
            .iter_mut()
            .find(|c| c.direction == link.direction && c.target_map == link.target)
            .context("installed city connection")?;
        ensure!(
            connection.offset == link.old_offset,
            "stale city connection offset"
        );
        connection.offset = link.offset;
        let mut neighbor = pack
            .data
            .maps
            .get(&link.target)
            .context("city neighbor")?
            .clone();
        let reverse_direction = opposite(&link.direction)?;
        let reciprocal = neighbor
            .attributes
            .connections
            .iter_mut()
            .find(|c| c.direction == reverse_direction && c.target_map == plan.city)
            .context("reciprocal city connection")?;
        ensure!(
            reciprocal.offset == -link.old_offset,
            "stale reciprocal city offset"
        );
        reciprocal.offset = link
            .offset
            .checked_neg()
            .context("connection offset overflow")?;
        authored_town_publication::replace(pack, neighbor)?;
    }
    Ok(())
}

pub(super) fn validate_connections(pack: &CompiledGamePack, plan: &Expansion) -> Result<()> {
    for link in &plan.connections {
        for lane in 0..2 {
            for (name, target, edge, expected, direction) in [
                (
                    plan.city.as_str(),
                    link.target.as_str(),
                    link.town_edge[lane],
                    link.route_edge[lane],
                    link.direction.as_str(),
                ),
                (
                    link.target.as_str(),
                    plan.city.as_str(),
                    link.route_edge[lane],
                    link.town_edge[lane],
                    opposite(&link.direction)?,
                ),
            ] {
                let module = &pack.data.maps[name];
                let native = OverworldMapData::from_attributes(
                    name,
                    &module.attributes,
                    module.blocks.clone(),
                );
                ensure!(
                    crate::authored_town_geometry::kind(pack, module, edge)? == "FLOOR",
                    "city seam must be ordinary dry floor"
                );
                let x = i16::try_from(edge[0])?;
                let y = i16::try_from(edge[1])?;
                let out = match direction {
                    "north" => TilePosition::new(x, y - 1),
                    "south" => TilePosition::new(x, y + 1),
                    "west" => TilePosition::new(x - 1, y),
                    "east" => TilePosition::new(x + 1, y),
                    _ => unreachable!(),
                };
                let transition = connection_for_tile_checked(&native, out)?
                    .context("city native seam projection")?;
                let destination = pack
                    .data
                    .resolve_connection_transition(&transition)?
                    .destination;
                ensure!(
                    destination.map_name == target
                        && destination.tile
                            == TilePosition::new(
                                i16::try_from(expected[0])?,
                                i16::try_from(expected[1])?
                            ),
                    "city native seam projects to wrong destination"
                );
            }
        }
    }
    Ok(())
}
