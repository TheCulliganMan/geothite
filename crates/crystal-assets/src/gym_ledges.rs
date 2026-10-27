//! Native one-way hops and useful escape loops in authored gym interiors.
use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::world::map::{Direction, OverworldMapData, TilePosition};
use crystal_core::world::movement::{
    LedgeJumpOutcome, OccupiedTile, PlayerMovementState, StepOptions, StepOutcome,
    attempt_ledge_jump_with_occupied_tiles, attempt_step_with_occupied_tiles,
};

use crate::authored_route_surface::walking_map_distances;
use crate::gym_design::Gym;
use crate::{CompiledGamePack, MapModule};

pub(crate) fn validate(
    pack: &CompiledGamePack,
    module: &MapModule,
    gym: &Gym,
    occupied: &BTreeSet<[u16; 2]>,
) -> Result<()> {
    if gym.ledges.is_empty() {
        return Ok(());
    }
    let map =
        OverworldMapData::from_attributes(&module.id, &module.attributes, module.blocks.clone());
    let collision = pack
        .data()
        .tileset_collision(&module.attributes.tileset_name)?;
    let tile = |[x, y]: [u16; 2]| -> Result<TilePosition> {
        Ok(TilePosition::new(i16::try_from(x)?, i16::try_from(y)?))
    };
    let bodies = occupied
        .iter()
        .map(|&at| {
            Ok(OccupiedTile {
                tile: tile(at)?,
                object_identifier: None,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    for ledge in &gym.ledges {
        let (direction, reverse) = match ledge.direction.as_str() {
            "south" => (Direction::Down, Direction::Up),
            "west" => (Direction::Left, Direction::Right),
            "east" => (Direction::Right, Direction::Left),
            "north" => (Direction::Up, Direction::Down),
            _ => anyhow::bail!("{}: unknown ledge direction", ledge.id),
        };
        let mut player = PlayerMovementState::new(tile(ledge.hop_from)?);
        ensure!(
            matches!(
                attempt_ledge_jump_with_occupied_tiles(
                    &mut player, direction, &map, &collision, StepOptions::default(), &bodies
                ),
                LedgeJumpOutcome::Jumped { to, .. } if to == tile(ledge.landing)?
            ),
            "{}: missing native hop or safe landing",
            ledge.id
        );
        player.facing = reverse;
        ensure!(
            matches!(
                attempt_step_with_occupied_tiles(
                    &mut player,
                    reverse,
                    &map,
                    &collision,
                    StepOptions::default(),
                    &bodies
                ),
                StepOutcome::Blocked { .. }
            ),
            "{}: ledge can be walked backward",
            ledge.id
        );
        let distances = walking_map_distances(pack, module, ledge.landing, occupied)?;
        ensure!(
            ledge.minimum_return_steps >= 6
                && distances
                    .get(&ledge.hop_from)
                    .is_some_and(|&steps| { steps >= u32::from(ledge.minimum_return_steps) }),
            "{}: return loop {:?} does not meet the declared minimum {}",
            ledge.id,
            distances.get(&ledge.hop_from),
            ledge.minimum_return_steps
        );
        let [x0, y0, x1, y1] = ledge.return_passage_core;
        ensure!(
            x0 < x1
                && y0 < y1
                && x1 <= module.attributes.width * 2
                && y1 <= module.attributes.height * 2,
            "{}: invalid return passage",
            ledge.id
        );
        let mut sealed = occupied.clone();
        sealed.extend((y0..y1).flat_map(|y| (x0..x1).map(move |x| [x, y])));
        ensure!(
            !sealed.contains(&ledge.landing) && !sealed.contains(&ledge.hop_from),
            "{}: return passage overlaps hop endpoints",
            ledge.id
        );
        ensure!(
            !walking_map_distances(pack, module, ledge.landing, &sealed)
                .with_context(|| format!("{}: sealed return", ledge.id))?
                .contains_key(&ledge.hop_from),
            "{}: alternate ascent bypasses the return passage",
            ledge.id
        );
    }
    Ok(())
}
