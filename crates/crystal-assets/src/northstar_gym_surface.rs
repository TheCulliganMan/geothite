//! Static resting-position edges for native Gym terrain, without a game session.

use std::collections::BTreeSet;

use anyhow::{Context, Result, ensure};
use crystal_core::world::collision::{TilesetCollision, permissions, sample_collision};
use crystal_core::world::map::{Direction, OverworldMapData, TilePosition};
use crystal_core::world::movement::{
    LedgeJumpOutcome, OccupiedTile, PlayerMovementState, StepOptions, StepOutcome,
    attempt_ledge_jump_with_occupied_tiles, attempt_step_with_occupied_tiles,
};

use crate::gym_design::Gym;
use crate::{CompiledGamePack, MapModule};

pub(crate) const DIRECTIONS: [Direction; 4] = [
    Direction::Up,
    Direction::Right,
    Direction::Down,
    Direction::Left,
];

pub(crate) struct Surface {
    map: OverworldMapData,
    collision: TilesetCollision,
    bodies: Vec<OccupiedTile>,
}

impl Surface {
    pub fn occupied(&self, at: [u16; 2]) -> Result<bool> {
        let at = tile(at)?;
        Ok(self.bodies.iter().any(|body| body.tile == at))
    }
    pub fn new(pack: &CompiledGamePack, room: &MapModule) -> Result<Self> {
        Ok(Self {
            map: OverworldMapData::from_attributes(&room.id, &room.attributes, room.blocks.clone()),
            collision: pack.data.tileset_collision(&room.attributes.tileset_name)?,
            bodies: room
                .objects
                .iter()
                .map(|actor| {
                    Ok(OccupiedTile {
                        tile: tile([actor.x, actor.y])?,
                        object_identifier: actor.object_identifier.clone(),
                    })
                })
                .collect::<Result<_>>()?,
        })
    }

    pub fn permission(&self, at: [u16; 2]) -> Result<u8> {
        Ok(sample_collision(&self.map, &self.collision, tile(at)?)
            .context("Gym tile outside authored bounds")?
            .permission)
    }

    pub fn edges(&self, at: [u16; 2], gym: &Gym) -> Result<Vec<[u16; 2]>> {
        let mut edges = Vec::new();
        for direction in DIRECTIONS {
            let Some(mut to) = self.step(at, direction)? else {
                continue;
            };
            let mut visited = BTreeSet::new();
            loop {
                ensure!(visited.insert(to), "cyclic forced Gym surface at {to:?}");
                match self.permission(to)? {
                    permissions::ICE | permissions::ICE_2B => {
                        if let Some(next) = self.step(to, direction)? {
                            to = next;
                        } else {
                            break;
                        }
                    }
                    permissions::WALK_LEFT | permissions::WALK_RIGHT => {
                        let direction = if self.permission(to)? == permissions::WALK_LEFT {
                            Direction::Left
                        } else {
                            Direction::Right
                        };
                        // Native authored wind is a direct step. Requiring the
                        // same edge to be safely walkable rejects wall/body hits.
                        to = self
                            .step(to, direction)?
                            .context("wind lane has no safe landing")?;
                    }
                    _ => break,
                }
            }
            for lift in &gym.lifts {
                if to == lift.ends_core[0] {
                    to = lift.ends_core[1];
                    break;
                }
                if to == lift.ends_core[1] {
                    to = lift.ends_core[0];
                    break;
                }
            }
            if !edges.contains(&to) {
                edges.push(to);
            }
        }
        Ok(edges)
    }

    fn step(&self, at: [u16; 2], direction: Direction) -> Result<Option<[u16; 2]>> {
        let mut position = PlayerMovementState::new(tile(at)?);
        let options = StepOptions {
            force_step_after_turn: true,
            ..Default::default()
        };
        if let LedgeJumpOutcome::Jumped { to, .. } = attempt_ledge_jump_with_occupied_tiles(
            &mut position,
            direction,
            &self.map,
            &self.collision,
            options,
            &self.bodies,
        ) {
            return Ok(Some(coordinate(to)?));
        }
        Ok(
            match attempt_step_with_occupied_tiles(
                &mut position,
                direction,
                &self.map,
                &self.collision,
                options,
                &self.bodies,
            ) {
                StepOutcome::Moved { to, .. } => Some(coordinate(to)?),
                _ => None,
            },
        )
    }
}

fn tile([x, y]: [u16; 2]) -> Result<TilePosition> {
    Ok(TilePosition::new(i16::try_from(x)?, i16::try_from(y)?))
}
fn coordinate(at: TilePosition) -> Result<[u16; 2]> {
    Ok([u16::try_from(at.x)?, u16::try_from(at.y)?])
}

pub(crate) fn adjacent(a: [u16; 2], b: [u16; 2]) -> bool {
    a[0].abs_diff(b[0]) + a[1].abs_diff(b[1]) == 1
}
