//! Native reciprocal lift pads and staged walking reachability in gym halls.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::world::collision::{permissions, sample_collision};
use crystal_core::world::map::{OverworldMapData, TilePosition};

use crate::authored_route_surface::walking_map_distances;
use crate::gym_design::Gym;
use crate::{CompiledGamePack, MapModule};

pub(crate) fn validate(
    pack: &CompiledGamePack,
    module: &MapModule,
    gym: &Gym,
    occupied: &BTreeSet<[u16; 2]>,
) -> Result<()> {
    let map =
        OverworldMapData::from_attributes(&module.id, &module.attributes, module.blocks.clone());
    let collision = pack
        .data()
        .tileset_collision(&module.attributes.tileset_name)?;
    let permission = |[x, y]: [u16; 2]| -> Result<u8> {
        Ok(sample_collision(
            &map,
            &collision,
            TilePosition::new(i16::try_from(x)?, i16::try_from(y)?),
        )
        .context("lift coordinate outside room")?
        .permission)
    };
    let mut pads = BTreeSet::new();
    for (index, lift) in gym.lifts.iter().enumerate() {
        for (side, at) in lift.ends_core.iter().enumerate() {
            ensure!(
                permission(*at)? == permissions::WARP_PANEL,
                "{}: {} is not a native lift pad at {at:?}",
                gym.id,
                lift.id
            );
            let warp = module
                .events
                .warps
                .get(3 + index * 2 + side)
                .context("lift warp record missing")?;
            ensure!(
                [warp.x, warp.y] == *at
                    && usize::try_from(warp.target_warp_id)? == 4 + index * 2 + (1 - side)
                    && warp.target_map_constant == gym.constant,
                "{}: lift must pair reciprocal native warp records",
                lift.id
            );
            let [x, y] = *at;
            ensure!(
                [
                    x.checked_sub(1).map(|x| [x, y]),
                    y.checked_sub(1).map(|y| [x, y]),
                    (x + 1 < module.attributes.width * 2).then_some([x + 1, y]),
                    (y + 1 < module.attributes.height * 2).then_some([x, y + 1]),
                ]
                .into_iter()
                .flatten()
                .any(|neighbor| {
                    !occupied.contains(&neighbor)
                        && permission(neighbor).ok() == Some(permissions::FLOOR)
                }),
                "{}: lift has no safe dismount at {at:?}",
                lift.id
            );
            pads.insert(*at);
        }
    }
    for y in 0..module.attributes.height * 2 {
        for x in 0..module.attributes.width * 2 {
            ensure!(
                permission([x, y])? != permissions::WARP_PANEL || pads.contains(&[x, y]),
                "{}: unbound warp pad at ({x},{y})",
                gym.id
            );
        }
    }
    Ok(())
}

/// A publication graph only: actual lift travel still needs controller proof.
pub(crate) fn reachable(
    pack: &CompiledGamePack,
    module: &MapModule,
    gym: &Gym,
    occupied: &BTreeSet<[u16; 2]>,
) -> Result<BTreeSet<[u16; 2]>> {
    if !gym.winds.is_empty() {
        ensure!(
            gym.lifts.is_empty(),
            "wind bridges cannot contain lift pads"
        );
        return crate::gym_wind::reachable(pack, module, gym, occupied);
    }
    let mut reached = walking_map_distances(pack, module, gym.arrival_core, occupied)?
        .into_keys()
        .collect::<BTreeSet<_>>();
    let mut expanded = BTreeSet::new();
    loop {
        let mut arrivals = Vec::new();
        for lift in &gym.lifts {
            for side in 0..2 {
                let from = lift.ends_core[side];
                if reached.contains(&from) && expanded.insert(from) {
                    arrivals.push(lift.ends_core[1 - side]);
                }
            }
        }
        if arrivals.is_empty() {
            break;
        }
        for at in arrivals {
            reached.extend(walking_map_distances(pack, module, at, occupied)?.into_keys());
        }
    }
    Ok(reached)
}
