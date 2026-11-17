//! Static native-collision graphs; no session, controller, save or game inputs.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::{Context, Result, ensure};
use crystal_core::world::collision::{
    Terrain, TilesetCollision, describe_collision, permissions, sample_collision,
};
use crystal_core::world::map::{Direction, OverworldMapData, TilePosition};
use crystal_core::world::movement::{
    LedgeJumpOutcome, MovementMode, OccupiedTile, PlayerMovementState, StepOptions, StepOutcome,
    attempt_ledge_jump_with_occupied_tiles, attempt_step_with_occupied_tiles,
};

use crate::northstar_cave_design::{Cave, HiddenItemSurface};
use crate::{CompiledGamePack, MapModule};

const DIRECTIONS: [Direction; 4] = [
    Direction::Up,
    Direction::Right,
    Direction::Down,
    Direction::Left,
];
type Position = [u16; 2];
type Graph = BTreeMap<Position, Vec<Position>>;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct TrainerPlacement {
    at: Position,
    defeated: bool,
}

type Placements = BTreeMap<String, TrainerPlacement>;

struct Encounter {
    id: String,
    origin: Position,
    direction: Direction,
    radius: u16,
}

struct Reachability {
    dry: Graph,
    water: Graph,
}

struct Surface {
    map: OverworldMapData,
    collision: TilesetCollision,
    bodies: Vec<OccupiedTile>,
}

impl Surface {
    fn new(
        pack: &CompiledGamePack,
        room: &MapModule,
        collected: bool,
        placements: &Placements,
    ) -> Result<Self> {
        Ok(Self {
            map: OverworldMapData::from_attributes(&room.id, &room.attributes, room.blocks.clone()),
            collision: pack.data.tileset_collision(&room.attributes.tileset_name)?,
            bodies: room
                .objects
                .iter()
                .filter(|actor| !collected || actor.object_type != "OBJECTTYPE_ITEMBALL")
                .map(|actor| {
                    Ok(OccupiedTile {
                        tile: tile(actor_position(
                            actor.object_identifier.as_deref(),
                            [actor.x, actor.y],
                            placements,
                        ))?,
                        object_identifier: actor.object_identifier.clone(),
                    })
                })
                .collect::<Result<_>>()?,
        })
    }

    fn occupied(&self, at: Position) -> Result<bool> {
        let at = tile(at)?;
        Ok(self.bodies.iter().any(|body| body.tile == at))
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

/// Reachable stopping squares from one entrance with the room's actual actor bodies.
pub(crate) fn resting_tiles(
    pack: &CompiledGamePack,
    room: &MapModule,
    entry: [u16; 2],
    surf: bool,
) -> Result<BTreeSet<[u16; 2]>> {
    let mut surface = Surface::new(pack, room, false, &Placements::new())?;
    // This graph ends at the room boundary. Actual connected-map steps carry
    // signed coordinates into the next map and are not local resting squares.
    surface.map.connections.clear();
    Ok(surface.graph(entry, surf)?.into_keys().collect())
}

pub(crate) fn validate(pack: &CompiledGamePack, room: &MapModule, cave: &Cave) -> Result<()> {
    let encounters = encounters(room, cave)?;
    let configurations = configurations(&encounters)?;
    for collected in [false, true] {
        let mut checked = BTreeMap::new();
        for placements in &configurations {
            let surface = Surface::new(pack, room, collected, placements)?;
            let reachability = validate_configuration(&surface, room, cave, placements)
                .with_context(|| format!("cave actor arrangement {placements:?}"))?;
            checked.insert(placements.clone(), reachability);
        }
        // Every geometric approach endpoint is included, even if another actor
        // initially conceals its trigger. This closes newly exposed sightlines.
        // Defeated actors retain their bodies, including adjacent victories that
        // leave the trainer at its original tile; they no longer own sight input.
        for placements in &configurations {
            let surface = Surface::new(pack, room, collected, placements)?;
            let reachable = &checked[placements];
            for encounter in &encounters {
                if placements[&encounter.id].defeated {
                    continue;
                }
                let mut visible_targets = 0;
                for distance in 1..=encounter.radius {
                    let target = ray_point(encounter, distance)?;
                    let dry = reachable.dry.contains_key(&target);
                    if !dry && !reachable.water.contains_key(&target) {
                        continue;
                    }
                    visible_targets += 1;
                    // Explicit native overrides test alignment, not occlusion.
                    // Inspect the full segment for every reachable target rather
                    // than stopping at the first wall and hiding a bad far ray.
                    for offset in 1..=distance {
                        let at = ray_point(encounter, offset)?;
                        ensure!(
                            surface.permission(at)? == permissions::FLOOR
                                && !surface.occupied(at)?,
                            "cave sightline {} crosses terrain or another actor at {at:?}",
                            encounter.id
                        );
                    }
                    let mut after = placements.clone();
                    after.insert(
                        encounter.id.clone(),
                        TrainerPlacement {
                            at: ray_point(encounter, distance - 1)?,
                            defeated: true,
                        },
                    );
                    let result = checked
                        .get(&after)
                        .context("bounded cave approach result")?;
                    // Checking only the arrival's component misses a player
                    // stranded beyond the trainer's newly occupied bridge tile.
                    ensure_trigger_survives(target, dry, result)
                        .with_context(|| format!("cave trainer {} approach", encounter.id))?;
                }
                ensure!(
                    visible_targets > 0,
                    "active cave trainer {} has no reachable sight target",
                    encounter.id
                );
            }
        }
    }
    Ok(())
}

fn encounters(room: &MapModule, cave: &Cave) -> Result<Vec<Encounter>> {
    let mut encounters = Vec::new();
    for actor in room
        .objects
        .iter()
        .filter(|actor| actor.object_type == "OBJECTTYPE_TRAINER")
    {
        let id = actor
            .object_identifier
            .as_deref()
            .context("stable cave trainer identity")?;
        let (direction, radius) =
            if let Some(trainer) = cave.trainers.iter().find(|trainer| trainer.id == id) {
                (trainer.direction.as_str(), trainer.radius)
            } else {
                let sight = cave
                    .retained_sightlines
                    .get(id)
                    .context("declared retained cave trainer sightline")?;
                (sight.direction.as_str(), sight.radius)
            };
        ensure!(
            (1..=3).contains(&radius),
            "bounded native cave trainer radius"
        );
        let facing = direction.to_ascii_uppercase();
        ensure!(
            actor.radius == radius
                && actor.sightline_direction_override.as_deref() == Some(facing.as_str())
                && actor.spritemovedata == format!("SPRITEMOVEDATA_STANDING_{facing}")
                && actor.move_range_x == 0
                && actor.move_range_y == 0,
            "cave trainer {id} must retain its explicit fixed native sightline"
        );
        ensure!(
            room.scripts
                .get(&actor.script)
                .and_then(|script| script.as_array())
                .and_then(|commands| commands.first())
                .is_some_and(|command| command["command"] == "trainer"),
            "cave battle must use native trainer program"
        );
        encounters.push(Encounter {
            id: id.into(),
            origin: [actor.x, actor.y],
            direction: match direction {
                "up" => Direction::Up,
                "right" => Direction::Right,
                "down" => Direction::Down,
                "left" => Direction::Left,
                _ => anyhow::bail!("cardinal cave trainer direction"),
            },
            radius,
        });
    }
    ensure!(
        encounters.len() == cave.trainers.len() + cave.retained_sightlines.len(),
        "every authored cave trainer contract resolves to a native actor"
    );
    Ok(encounters)
}

fn configurations(encounters: &[Encounter]) -> Result<Vec<Placements>> {
    let mut configurations = vec![Placements::new()];
    for encounter in encounters {
        let mut choices = vec![TrainerPlacement {
            at: encounter.origin,
            defeated: false,
        }];
        for offset in 0..encounter.radius {
            choices.push(TrainerPlacement {
                at: ray_point(encounter, offset)?,
                defeated: true,
            });
        }
        ensure!(
            configurations.len().saturating_mul(choices.len()) <= 256,
            "cave trainer approach combinations exceed the static bound"
        );
        let mut expanded = Vec::new();
        for configuration in configurations {
            for choice in &choices {
                let mut next = configuration.clone();
                next.insert(encounter.id.clone(), choice.clone());
                expanded.push(next);
            }
        }
        configurations = expanded;
    }
    Ok(configurations)
}

fn ray_point(encounter: &Encounter, distance: u16) -> Result<Position> {
    let [x, y] = encounter.origin;
    let [dx, dy] = match encounter.direction {
        Direction::Up => [0, -1],
        Direction::Right => [1, 0],
        Direction::Down => [0, 1],
        Direction::Left => [-1, 0],
    };
    Ok([
        u16::try_from(i32::from(x) + dx * i32::from(distance))?,
        u16::try_from(i32::from(y) + dy * i32::from(distance))?,
    ])
}

fn actor_position(id: Option<&str>, authored: Position, placements: &Placements) -> Position {
    id.and_then(|id| placements.get(id))
        .map_or(authored, |placement| placement.at)
}

fn ensure_trigger_survives(target: Position, dry: bool, result: &Reachability) -> Result<()> {
    let graph = if dry { &result.dry } else { &result.water };
    ensure!(
        graph.contains_key(&target),
        "trainer approach strands the player at {target:?}"
    );
    Ok(())
}

fn validate_configuration(
    surface: &Surface,
    room: &MapModule,
    cave: &Cave,
    placements: &Placements,
) -> Result<Reachability> {
    let occupied: BTreeSet<_> = surface
        .bodies
        .iter()
        .map(|body| (body.tile.x, body.tile.y))
        .collect();
    ensure!(
        occupied.len() == surface.bodies.len(),
        "cave actor approach bodies overlap"
    );
    ensure!(
        surface.permission(cave.arrival)? == permissions::FLOOR
            && !surface.occupied(cave.arrival)?,
        "safe cave migration arrival"
    );
    let graph = surface.graph(cave.arrival, false)?;
    let water = surface.graph(cave.arrival, true)?;
    ensure_return_paths(&graph, cave.arrival)?;
    ensure_return_paths(&water, cave.arrival)?;
    for waypoint in &cave.waypoints {
        let occupied_by_defeated_trainer = placements
            .values()
            .any(|placement| placement.defeated && placement.at == *waypoint);
        ensure!(
            graph.contains_key(waypoint)
                || (occupied_by_defeated_trainer
                    && graph.keys().any(|&from| adjacent(from, *waypoint))),
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
        let at = actor_position(
            actor.object_identifier.as_deref(),
            [actor.x, actor.y],
            placements,
        );
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
            water.keys().any(|&from| adjacent(from, at))
                && (event.event_type != "BGEVENT_ITEM" || !surface.occupied(at)?),
            "cave background event lacks approach: {}",
            event.script
        );
    }
    for item in &cave.hidden_items {
        let dry_approach = graph.keys().any(|&from| adjacent(from, item.tile));
        let approach = if item.requires_surf {
            !dry_approach && water.keys().any(|&from| adjacent(from, item.tile))
        } else {
            dry_approach
        };
        let permission = match item.target_surface {
            HiddenItemSurface::Floor => permissions::FLOOR,
            // A checks the facing square. The player stands beside the rock,
            // so the target itself is deliberately impassable.
            HiddenItemSurface::Rock => permissions::WALL,
        };
        ensure!(
            surface.permission(item.tile)? == permission
                && !surface.occupied(item.tile)?
                && approach,
            "hidden cave discovery violates its dry/Surf approach contract: {}",
            item.id
        );
    }
    for &[x, y] in water.keys() {
        ensure!(
            describe_collision(surface.permission([x, y])?).terrain != Terrain::Water
                || (x > 0
                    && y > 0
                    && x + 1 < room.attributes.width * 2
                    && y + 1 < room.attributes.height * 2)
                || room
                    .events
                    .warps
                    .iter()
                    .any(|warp| [warp.x, warp.y] == [x, y]),
            "cave Surf course reaches an unconnected map boundary"
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
    Ok(Reachability { dry: graph, water })
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
    use crystal_core::world::map::Direction;

    use super::{
        Encounter, Graph, Reachability, configurations, ensure_return_paths,
        ensure_trigger_survives,
    };

    #[test]
    fn cave_return_graph_rejects_a_one_way_trap() {
        let trapped = Graph::from([([0, 0], vec![[1, 0]]), ([1, 0], vec![])]);
        assert!(ensure_return_paths(&trapped, [0, 0]).is_err());
        let circuit = Graph::from([([0, 0], vec![[1, 0]]), ([1, 0], vec![[0, 0]])]);
        ensure_return_paths(&circuit, [0, 0]).expect("return circuit");
    }

    #[test]
    fn cave_approach_cannot_rescue_a_stranded_walker_by_assuming_surf() {
        let after = Reachability {
            dry: Graph::from([([0, 0], vec![])]),
            water: Graph::from([([0, 0], vec![[2, 0]]), ([2, 0], vec![[0, 0]])]),
        };
        assert!(ensure_trigger_survives([2, 0], true, &after).is_err());
        ensure_trigger_survives([2, 0], false, &after).expect("same Surf capability returns");
        assert!(ensure_trigger_survives([3, 0], false, &after).is_err());
    }

    #[test]
    fn cave_approach_includes_unmoved_victories_and_later_exposed_targets() {
        let encounter = Encounter {
            id: "NATIVE_TRAINER".into(),
            origin: [4, 4],
            direction: Direction::Down,
            radius: 3,
        };
        let states = configurations(&[encounter]).expect("bounded trainer positions");
        assert_eq!(states.len(), 4);
        assert!(states.iter().any(|state| {
            let trainer = &state["NATIVE_TRAINER"];
            trainer.at == [4, 4] && trainer.defeated
        }));
        // Do not filter this endpoint by original-body reachability: another
        // trainer moving can expose the distance-three trigger at (4,7).
        assert!(states.iter().any(|state| {
            let trainer = &state["NATIVE_TRAINER"];
            trainer.at == [4, 6] && trainer.defeated
        }));
    }
}
