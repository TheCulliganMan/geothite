//! Exact neighboring river patches and reciprocal native water projections.

use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::world::map::{OverworldMapData, TilePosition};
use crystal_core::world::session::connection_for_tile_checked;
use sha2::{Digest, Sha256};

use crate::authored_city_expansion_design::Expansion;
use crate::authored_city_recomposition_design::{NeighborTerrain, WaterSeam};
use crate::authored_town_geometry::kind;
use crate::{CompiledGamePack, MapModule};

pub(super) fn revise_neighbors(
    pack: &mut CompiledGamePack,
    plan: &Expansion,
    seams: &[WaterSeam],
    revisions: &[NeighborTerrain],
) -> Result<()> {
    let mut used = BTreeSet::new();
    for revision in revisions {
        ensure!(
            used.insert(revision.map.as_str())
                && !revision.changes.is_empty()
                && seams.iter().any(|seam| seam.target == revision.map)
                && plan
                    .connections
                    .iter()
                    .any(|link| link.target == revision.map),
            "river terrain revision needs a unique actual water neighbor"
        );
        let old = pack
            .data
            .maps
            .get(&revision.map)
            .context("river neighbor")?;
        let hash = format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(&serde_json::to_value(old)?)?)
        );
        ensure!(
            hash == revision.native_module_sha256,
            "river neighbor preimage differs"
        );
        let mut revised = old.clone();
        let mut cells = BTreeSet::new();
        for change in &revision.changes {
            let [x, y] = change.metatile;
            ensure!(
                x < old.attributes.width
                    && y < old.attributes.height
                    && cells.insert(change.metatile)
                    && change.after <= 255,
                "river neighbor block position is invalid/repeated"
            );
            let index = usize::from(y * old.attributes.width + x);
            ensure!(
                old.blocks[index] == change.before && change.before != change.after,
                "river neighbor block preimage differs or change is empty"
            );
            revised.blocks[index] = change.after;
        }
        crate::authored_town_publication::replace(pack, revised)?;
    }
    Ok(())
}

fn reverse(direction: &str) -> Result<&'static str> {
    Ok(match direction {
        "north" => "south",
        "south" => "north",
        "east" => "west",
        "west" => "east",
        _ => anyhow::bail!("invalid river seam direction"),
    })
}

fn outward(module: &MapModule, edge: [u16; 2], direction: &str) -> Result<TilePosition> {
    let [x, y] = edge;
    let width = module.attributes.width * 2;
    let height = module.attributes.height * 2;
    ensure!(x < width && y < height, "river edge outside native map");
    let boundary = match direction {
        "north" => y == 0,
        "south" => y + 1 == height,
        "west" => x == 0,
        "east" => x + 1 == width,
        _ => false,
    };
    ensure!(boundary, "river lane is not on its declared map edge");
    let x = i16::try_from(x)?;
    let y = i16::try_from(y)?;
    Ok(match direction {
        "north" => TilePosition::new(x, y - 1),
        "south" => TilePosition::new(x, y + 1),
        "west" => TilePosition::new(x - 1, y),
        "east" => TilePosition::new(x + 1, y),
        _ => unreachable!(),
    })
}

pub(super) fn validate_seams(
    pack: &CompiledGamePack,
    plan: &Expansion,
    seams: &[WaterSeam],
) -> Result<BTreeSet<[u16; 2]>> {
    let city = pack
        .data
        .maps
        .get(&plan.city)
        .context("recomposed river city")?;
    let mut edges = BTreeSet::new();
    let mut directions = BTreeSet::new();
    for seam in seams {
        ensure!(
            directions.insert(seam.direction.as_str())
                && !seam.town_edge.is_empty()
                && seam.town_edge.len() == seam.route_edge.len()
                && city
                    .attributes
                    .connections
                    .iter()
                    .any(|link| link.direction == seam.direction && link.target_map == seam.target),
            "water seam needs unique paired lanes on an installed connection"
        );
        for (&town, &route) in seam.town_edge.iter().zip(&seam.route_edge) {
            ensure!(edges.insert(town), "duplicate city river lane");
            for (name, target, edge, expected, direction) in [
                (
                    plan.city.as_str(),
                    seam.target.as_str(),
                    town,
                    route,
                    seam.direction.as_str(),
                ),
                (
                    seam.target.as_str(),
                    plan.city.as_str(),
                    route,
                    town,
                    reverse(&seam.direction)?,
                ),
            ] {
                let module = pack.data.maps.get(name).context("water seam map")?;
                ensure!(
                    kind(pack, module, edge)? == "WATER",
                    "river seam must retain real Water collision"
                );
                let native = OverworldMapData::from_attributes(
                    name,
                    &module.attributes,
                    module.blocks.clone(),
                );
                let transition =
                    connection_for_tile_checked(&native, outward(module, edge, direction)?)?
                        .context("river native seam projection")?;
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
                    "river native seam projects to a different tile/map"
                );
            }
        }
    }
    Ok(edges)
}
