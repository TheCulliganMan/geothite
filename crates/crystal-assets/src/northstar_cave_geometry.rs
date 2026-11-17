//! Static native-collision graphs; no session, controller, save or game inputs.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use crystal_core::world::collision::{TilesetCollision, permissions, sample_collision};
use crystal_core::world::map::{Direction, OverworldMapData, TilePosition};
use crystal_core::world::movement::{
    LedgeJumpOutcome, MovementMode, OccupiedTile, PlayerMovementState, StepOptions, StepOutcome,
    attempt_ledge_jump_with_occupied_tiles, attempt_step_with_occupied_tiles,
};

use crate::northstar_cave_design::Cave;
use crate::{CompiledGamePack, MapModule};

const DIRECTIONS: [Direction; 4] = [
    Direction::Up,
    Direction::Right,
    Direction::Down,
    Direction::Left,
];
type Position = [u16; 2];
type Graph = BTreeMap<Position, Vec<Position>>;

struct Surface {
    map: OverworldMapData,
    collision: TilesetCollision,
    bodies: Vec<OccupiedTile>,
}

impl Surface {
    fn new(pack: &CompiledGamePack, room: &MapModule, collected: bool) -> Result<Self> {
        Ok(Self {
            map: OverworldMapData::from_attributes(&room.id, &room.attributes, room.blocks.clone()),
            collision: pack.data.tileset_collision(&room.attributes.tileset_name)?,
            bodies: room
                .objects
                .iter()
                .filter(|actor| !collected || actor.object_type != "OBJECTTYPE_ITEMBALL")
                .map(|actor| {
                    Ok(OccupiedTile {
                        tile: tile([actor.x, actor.y])?,
                        object_identifier: actor.object_identifier.clone(),
                    })
                })
                .collect::<Result<_>>()?,
        })
    }

    fn permission(&self, at: Position) -> Result<u8> {
        Ok(sample_collision(&self.map, &self.collision, tile(at)?)
            .context("cave coordinate outside map")?
            .permission)
    }

    fn step(&self, at: Position, direction: Direction, water: bool) -> Result<Option<Position>> {
        let mode = if water {
            MovementMode::Surf
        } else {
            MovementMode::Normal
        };
        let mut state = PlayerMovementState::new(tile(at)?).with_mode(mode);
        let options = StepOptions {
            force_step_after_turn: true,
            ..Default::default()
        };
        if let LedgeJumpOutcome::Jumped { to, .. } = attempt_ledge_jump_with_occupied_tiles(
            &mut state,
            direction,
            &self.map,
            &self.collision,
            options,
            &self.bodies,
        ) {
            return Ok(Some(position(to)?));
        }
        Ok(
            match attempt_step_with_occupied_tiles(
                &mut state,
                direction,
                &self.map,
                &self.collision,
                options,
                &self.bodies,
            ) {
                StepOutcome::Moved { to, .. } => Some(position(to)?),
                _ => None,
            },
        )
    }

    fn edges(&self, at: Position, water: bool) -> Result<Vec<Position>> {
        let mut edges = Vec::new();
        for direction in DIRECTIONS {
            let Some(mut to) = self.step(at, direction, water)? else {
                continue;
            };
            let mut visited = BTreeSet::new();
            while matches!(self.permission(to)?, permissions::ICE | permissions::ICE_2B) {
                ensure!(visited.insert(to), "cyclic forced ice slide");
                let Some(next) = self.step(to, direction, water)? else {
                    break;
                };
                to = next;
            }
            if !edges.contains(&to) {
                edges.push(to);
            }
        }
        Ok(edges)
    }

    fn graph(&self, entry: Position, water: bool) -> Result<Graph> {
        let mut graph = BTreeMap::new();
        let mut queued = BTreeSet::from([entry]);
        let mut todo = VecDeque::from([entry]);
        while let Some(at) = todo.pop_front() {
            let edges = self.edges(at, water)?;
            for &next in &edges {
                if queued.insert(next) {
                    todo.push_back(next);
                }
            }
            graph.insert(at, edges);
        }
        Ok(graph)
    }
}

pub(crate) fn validate(pack: &CompiledGamePack, room: &MapModule, cave: &Cave) -> Result<()> {
    for collected in [false, true] {
        let surface = Surface::new(pack, room, collected)?;
        ensure!(
            surface.permission(cave.arrival)? == permissions::FLOOR
                && !room
                    .objects
                    .iter()
                    .any(|actor| [actor.x, actor.y] == cave.arrival),
            "safe cave migration arrival"
        );
        let graph = surface.graph(cave.arrival, false)?;
        let water = surface.graph(cave.arrival, true)?;
        ensure_return_paths(&graph, cave.arrival)?;
        ensure_return_paths(&water, cave.arrival)?;
        for waypoint in &cave.waypoints {
            ensure!(
                graph.contains_key(waypoint),
                "unreachable cave landmark {waypoint:?}"
            );
        }
        for waypoint in &cave.optional_water_waypoints {
            ensure!(
                water.contains_key(waypoint) && !graph.contains_key(waypoint),
                "optional island must require Surf: {waypoint:?}"
            );
        }
        for actor in &room.objects {
            let at = [actor.x, actor.y];
            ensure!(
                surface.permission(at)? == permissions::FLOOR
                    && graph.keys().any(|&from| adjacent(from, at)),
                "cave actor lacks dry reachable approach: {:?}",
                actor.object_identifier
            );
        }
        for warp in &room.events.warps {
            ensure!(
                graph.contains_key(&[warp.x, warp.y]),
                "unreachable retained cave warp {}",
                warp.index
            );
        }
        for event in &room.events.bg_events {
            let at = [event.x, event.y];
            ensure!(
                water.keys().any(|&from| adjacent(from, at)),
                "cave background event lacks approach: {}",
                event.script
            );
        }
        for item in &cave.hidden_items {
            ensure!(
                surface.permission(item.tile)? == permissions::FLOOR
                    && graph.keys().any(|&from| adjacent(from, item.tile)),
                "hidden cave discovery lacks resting approach: {}",
                item.id
            );
        }
        for actor in room
            .objects
            .iter()
            .filter(|actor| actor.object_type == "OBJECTTYPE_TRAINER")
        {
            ensure!(actor.radius > 0, "cave trainers battle on sight");
            let direction = match actor.spritemovedata.as_str() {
                "SPRITEMOVEDATA_STANDING_UP" => Direction::Up,
                "SPRITEMOVEDATA_STANDING_RIGHT" => Direction::Right,
                "SPRITEMOVEDATA_STANDING_DOWN" => Direction::Down,
                "SPRITEMOVEDATA_STANDING_LEFT" => Direction::Left,
                _ => anyhow::bail!("fixed native cave trainer facing required"),
            };
            let front = surface
                .step([actor.x, actor.y], direction, false)?
                .context("cave trainer faces blocked terrain")?;
            ensure!(
                graph.contains_key(&front),
                "cave trainer sightline cannot reach the player"
            );
            ensure!(
                room.scripts
                    .get(&actor.script)
                    .and_then(|script| script.as_array())
                    .and_then(|commands| commands.first())
                    .is_some_and(|command| command["command"] == "trainer"),
                "cave battle must use native trainer program"
            );
        }
        for trainer in &cave.trainers {
            let [x, y] = trainer.tile;
            let delta = match trainer.direction.as_str() {
                "up" => [0_i32, -1],
                "right" => [1, 0],
                "down" => [0, 1],
                "left" => [-1, 0],
                _ => anyhow::bail!("native trainer direction"),
            };
            let mut visible_targets = 0;
            for distance in 1..=trainer.radius {
                let point = |step: u16| -> Result<Position> {
                    Ok([
                        u16::try_from(i32::from(x) + delta[0] * i32::from(step))?,
                        u16::try_from(i32::from(y) + delta[1] * i32::from(step))?,
                    ])
                };
                let target = point(distance)?;
                if !graph.contains_key(&target) {
                    continue;
                }
                visible_targets += 1;
                // The fixed native sightline checks alignment and radius, not
                // intervening geometry. Certify every hittable ray target so
                // a wall cannot create an implausible through-rock encounter.
                for step in 1..=distance {
                    let at = point(step)?;
                    let native = tile(at)?;
                    ensure!(
                        surface.permission(at)? == permissions::FLOOR
                            && !surface.bodies.iter().any(|body| body.tile == native),
                        "cave sightline crosses rock or another actor"
                    );
                }
            }
            ensure!(
                visible_targets > 0,
                "new cave trainer has no reachable sight target"
            );
        }
        for y in 0..room.attributes.height * 2 {
            for x in 0..room.attributes.width * 2 {
                let permission = surface.permission([x, y])?;
                ensure!(
                    permission != permissions::PIT,
                    "unpaired cave fall not supported"
                );
                if permission == permissions::LADDER {
                    ensure!(
                        room.events
                            .warps
                            .iter()
                            .any(|warp| [warp.x, warp.y] == [x, y]),
                        "native cave ladder lacks retained warp"
                    );
                }
            }
        }
    }
    Ok(())
}

fn ensure_return_paths(graph: &Graph, entry: Position) -> Result<()> {
    let mut reverse: Graph = graph.keys().map(|&at| (at, Vec::new())).collect();
    for (&from, edges) in graph {
        for to in edges {
            reverse
                .get_mut(to)
                .context("complete cave graph")?
                .push(from);
        }
    }
    let mut seen = BTreeSet::from([entry]);
    let mut todo = VecDeque::from([entry]);
    while let Some(at) = todo.pop_front() {
        for &next in &reverse[&at] {
            if seen.insert(next) {
                todo.push_back(next);
            }
        }
    }
    ensure!(
        seen.len() == graph.len(),
        "cave contains a reachable trap without an exit route"
    );
    Ok(())
}

fn adjacent(a: Position, b: Position) -> bool {
    a[0].abs_diff(b[0]) + a[1].abs_diff(b[1]) == 1
}
fn tile([x, y]: Position) -> Result<TilePosition> {
    Ok(TilePosition::new(i16::try_from(x)?, i16::try_from(y)?))
}
fn position(at: TilePosition) -> Result<Position> {
    Ok([u16::try_from(at.x)?, u16::try_from(at.y)?])
}

#[cfg(test)]
mod tests {
    use super::{Graph, ensure_return_paths};

    #[test]
    fn cave_return_graph_rejects_a_one_way_trap() {
        let trapped = Graph::from([([0, 0], vec![[1, 0]]), ([1, 0], vec![])]);
        assert!(ensure_return_paths(&trapped, [0, 0]).is_err());
        let circuit = Graph::from([([0, 0], vec![[1, 0]]), ([1, 0], vec![[0, 0]])]);
        ensure_return_paths(&circuit, [0, 0]).expect("return circuit");
    }
}
