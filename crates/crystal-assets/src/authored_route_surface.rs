//! Static authored-surface reachability using the production numeric permissions.
//!
//! This is a publication safety graph, not a second input dispatcher. Surf/Cut
//! prerequisites are assumed available. Native entry/departure masks, Surf's
//! blocked mount permissions, occupied cells and native ledge hops are honored.
//! Warps are treated as passable surface cells without following their targets;
//! currents/forced walking do not restrict later graph choices. Those choices
//! deliberately over-approximate local reachability and still need controller
//! review. Interior warp networks, earned flags, NPC movement and field-move
//! presentation are outside this static check.
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap};

use anyhow::{Context, Result, ensure};
use crystal_core::world::collision::{
    PlayerTraversalState, Terrain, TilesetCollision, describe_collision,
    is_direction_blocked_leaving, is_permission_passable, sample_collision,
};
use crystal_core::world::map::{Direction, OverworldMapData, TilePosition};
use crystal_core::world::movement::{
    LedgeJumpOutcome, MovementMode, OccupiedTile, PlayerMovementState, StepOptions,
    attempt_ledge_jump_with_occupied_tiles,
};

use crate::authored_route_design::RouteDesign;
use crate::{CompiledGamePack, MapModule};

const DIRECTIONS: [Direction; 4] = [
    Direction::Up,
    Direction::Down,
    Direction::Left,
    Direction::Right,
];

/// Returns ordinary walking/Surf cells reachable from the first route arrival.
///
/// Uses the supplied tileset's numeric permission table, including WATER_21 and
/// directional buoy variants. A land-to-water edge additionally respects the
/// actual native Surf rule's blocked mount permissions. Occupancy is honored
/// exactly; collectible removal is the caller's explicit choice. The authored
/// whole has no barriers at internal chunk seams: reciprocal native neighbors
/// must be verified separately by the publisher.
///
/// # Errors
/// Returns an error for invalid extents, source collision, arrival or occupancy.
pub(crate) fn reachable_with_surf(
    pack: &CompiledGamePack,
    design: &RouteDesign,
    blocks: &[u16],
    tileset: &str,
    occupied: &BTreeSet<[u16; 2]>,
) -> Result<BTreeSet<[u16; 2]>> {
    let surface = Surface::new(pack, design, blocks, tileset)?;
    surface.reachable(arrival(design)?, occupied)
}

/// Returns local walking/Surf reachability on an explicitly supplied native map.
///
/// Uses the same numeric permissions, ledges, mount rules and occupancy as the
/// route check. Connections and warp destinations are not followed. Callers can
/// conservatively block a complete guard plane to detect walk/Surf bypasses;
/// actual conditional scripts and movement still require controller proof.
///
/// # Errors
/// Returns an error for invalid extents, source collision, start or occupancy.
pub(crate) fn reachable_map_with_surf(
    pack: &CompiledGamePack,
    module: &MapModule,
    start: [u16; 2],
    occupied: &BTreeSet<[u16; 2]>,
) -> Result<BTreeSet<[u16; 2]>> {
    Surface::from_blocks(
        pack,
        &module.id,
        [module.attributes.width, module.attributes.height],
        &module.blocks,
        &module.attributes.tileset_name,
    )?
    .reachable(start, occupied)
}

/// Returns local walking distances using native entry, departure and ledge rules.
///
/// Hops cost two tiles. Water, warp destinations and forced-surface presentation
/// are outside this graph; earned movement still needs controller verification.
///
/// # Errors
/// Returns an error for invalid geometry, source collision, start or occupancy.
pub(crate) fn walking_map_distances(
    pack: &CompiledGamePack,
    module: &MapModule,
    start: [u16; 2],
    occupied: &BTreeSet<[u16; 2]>,
) -> Result<BTreeMap<[u16; 2], u32>> {
    Surface::from_blocks(
        pack,
        &module.id,
        [module.attributes.width, module.attributes.height],
        &module.blocks,
        &module.attributes.tileset_name,
    )?
    .shortest_paths(start, occupied, false, None)
}

/// Returns the shortest geometric tile distance between explicit route points.
///
/// Uses exactly the same numeric/native edge rules as `reachable_with_surf`.
/// Ordinary moves and Surf mounts/dismounts cost one tile; a native two-tile hop
/// costs two, even though it uses one input. `allow_surf=false` excludes water.
/// Occupied cells remain blocked. `None` means the target has no legal path.
///
/// # Errors
/// Returns an error for invalid geometry, coordinates, occupancy or start mode.
#[allow(
    clippy::too_many_arguments,
    reason = "Publication queries keep source geometry, occupancy and both endpoints explicit."
)]
pub(crate) fn shortest_path_distance(
    pack: &CompiledGamePack,
    design: &RouteDesign,
    blocks: &[u16],
    tileset: &str,
    occupied: &BTreeSet<[u16; 2]>,
    from: [u16; 2],
    to: [u16; 2],
    allow_surf: bool,
) -> Result<Option<u32>> {
    let surface = Surface::new(pack, design, blocks, tileset)?;
    surface.permission(to)?;
    Ok(surface
        .shortest_paths(from, occupied, allow_surf, Some(to))?
        .get(&to)
        .copied())
}

/// Rejects a reachable unclosed water edge after declared Cut gates are opened.
///
/// Uses actual source replacement rules, including the legacy shortcut; it
/// never substitutes a guessed Floor block. All actor bodies are cleared for
/// this eventual-access audit: collection or trainer/NPC movement can vacate a
/// currently occupied tile. This conservative safety assumption does not change
/// occupancy in reusable reachability/distance calls. Numeric water behind a sealed
/// fence is not an escape. An outer directional buoy is accepted only where
/// its real departure mask closes every outward direction at that cell.
///
/// This checks the global route perimeter, not internal chunk seams. It does
/// not follow interior warps; native neighbor/room validation remains separate.
///
/// # Errors
/// Returns an error for invalid geometry, undeclared native Cut replacement,
/// conflicting declared replacement or a reachable outward water permission.
pub(crate) fn validate_outer_boundary(
    pack: &CompiledGamePack,
    design: &RouteDesign,
    blocks: &[u16],
    tileset: &str,
    _occupied: &BTreeSet<[u16; 2]>,
) -> Result<()> {
    let opened = opened_cut_blocks(pack, design, blocks, tileset)?;
    let surface = Surface::new(pack, design, &opened, tileset)?;
    let seen = surface.reachable(arrival(design)?, &BTreeSet::new())?;
    surface.validate_outer(&seen)
}

fn arrival(design: &RouteDesign) -> Result<[u16; 2]> {
    let [x, y] = design
        .endpoints
        .first()
        .context("route has no arrival")?
        .gate_metatile;
    Ok([
        x.checked_mul(2).context("arrival x overflow")?,
        y.checked_mul(2).context("arrival y overflow")?,
    ])
}

fn opened_cut_blocks(
    pack: &CompiledGamePack,
    design: &RouteDesign,
    blocks: &[u16],
    tileset: &str,
) -> Result<Vec<u16>> {
    let source = Surface::new(pack, design, blocks, tileset)?;
    let mut opened = blocks.to_vec();
    let mut gates = design
        .cut_gates
        .iter()
        .map(|gate| (gate.tile, Some(gate.replacement_block.as_str())))
        .collect::<Vec<_>>();
    if let Some(shortcut) = &design.shortcut {
        gates.push((shortcut.tile, None));
    }
    for (tile, declared) in gates {
        let permission = source.permission(tile)?;
        ensure!(
            pack.data()
                .field_moves
                .cut
                .target_collisions
                .contains(&permission),
            "declared Cut target has no native Cut permission"
        );
        let index = usize::from(tile[1] / 2) * usize::from(design.dimensions_metatiles[0])
            + usize::from(tile[0] / 2);
        let original = blocks[index];
        let replacement = pack
            .data()
            .field_moves
            .cut
            .replacements
            .get(tileset)
            .and_then(|table| table.get(&original))
            .context("declared Cut target has no native source replacement")?;
        ensure!(
            replacement.variant == "tree",
            "route Cut gate must use the native tree replacement"
        );
        if let Some(declared) = declared {
            ensure!(
                u16::from_str_radix(declared, 16).context("hexadecimal Cut replacement")?
                    == replacement.replacement_block_id,
                "declared Cut replacement differs from native source"
            );
        }
        opened[index] = replacement.replacement_block_id;
    }
    Ok(opened)
}

struct Surface {
    map: OverworldMapData,
    collision: TilesetCollision,
    blocked_mounts: Vec<u8>,
}

impl Surface {
    fn new(
        pack: &CompiledGamePack,
        design: &RouteDesign,
        blocks: &[u16],
        tileset: &str,
    ) -> Result<Self> {
        Self::from_blocks(
            pack,
            &design.id,
            design.dimensions_metatiles,
            blocks,
            tileset,
        )
    }

    fn from_blocks(
        pack: &CompiledGamePack,
        name: &str,
        [width, height]: [u16; 2],
        blocks: &[u16],
        tileset: &str,
    ) -> Result<Self> {
        let map = OverworldMapData {
            name: name.into(),
            width,
            height,
            border_block: 0,
            connections: Vec::new(),
            metatile_ids: blocks.to_vec(),
        };
        map.validate().map_err(anyhow::Error::msg)?;
        let (w, h) = map
            .checked_tile_bounds()
            .context("surface tile extent overflow")?;
        ensure!(
            w <= i16::MAX as u16 && h <= i16::MAX as u16,
            "surface extent exceeds native tile positions"
        );
        let collision = pack.data().tileset_collision(tileset)?;
        for block in blocks {
            ensure!(
                usize::from(*block) < collision.metatiles.len(),
                "source metatile collision is missing"
            );
        }
        Ok(Self {
            map,
            collision,
            blocked_mounts: pack.data().field_moves.surf.blocked_collisions.clone(),
        })
    }

    fn permission(&self, [x, y]: [u16; 2]) -> Result<u8> {
        let position = TilePosition::new(i16::try_from(x)?, i16::try_from(y)?);
        sample_collision(&self.map, &self.collision, position)
            .map(|sample| sample.permission)
            .context("surface position outside authored collision")
    }

    fn reachable(
        &self,
        start: [u16; 2],
        occupied: &BTreeSet<[u16; 2]>,
    ) -> Result<BTreeSet<[u16; 2]>> {
        ensure!(
            describe_collision(self.permission(start)?).terrain == Terrain::Land,
            "walking arrival is blocked"
        );
        Ok(self
            .shortest_paths(start, occupied, true, None)?
            .into_keys()
            .collect())
    }

    fn shortest_paths(
        &self,
        start: [u16; 2],
        occupied: &BTreeSet<[u16; 2]>,
        allow_surf: bool,
        stop: Option<[u16; 2]>,
    ) -> Result<BTreeMap<[u16; 2], u32>> {
        let (width, height) = self
            .map
            .checked_tile_bounds()
            .context("surface extent overflow")?;
        let terrain = describe_collision(self.permission(start)?).terrain;
        ensure!(
            (terrain == Terrain::Land || (allow_surf && terrain == Terrain::Water))
                && !occupied.contains(&start),
            "surface start is blocked for the requested traversal"
        );
        ensure!(
            occupied.iter().all(|[x, y]| *x < width && *y < height),
            "occupied cell lies outside authored surface"
        );
        let bodies = occupied
            .iter()
            .map(|[x, y]| {
                Ok(OccupiedTile {
                    tile: TilePosition::new(i16::try_from(*x)?, i16::try_from(*y)?),
                    object_identifier: None,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let mut distances = BTreeMap::from([(start, 0)]);
        let mut queue = BinaryHeap::from([Reverse((0_u32, start))]);
        while let Some(Reverse((cost, tile))) = queue.pop() {
            if distances.get(&tile) != Some(&cost) {
                continue;
            }
            if stop == Some(tile) {
                break;
            }
            for (target, steps) in self.edges(tile, occupied, &bodies, allow_surf)? {
                let next = cost
                    .checked_add(steps)
                    .context("surface distance overflow")?;
                if distances.get(&target).is_none_or(|old| next < *old) {
                    distances.insert(target, next);
                    queue.push(Reverse((next, target)));
                }
            }
        }
        Ok(distances)
    }

    fn edges(
        &self,
        tile: [u16; 2],
        occupied: &BTreeSet<[u16; 2]>,
        bodies: &[OccupiedTile],
        allow_surf: bool,
    ) -> Result<Vec<([u16; 2], u32)>> {
        let (width, height) = self
            .map
            .checked_tile_bounds()
            .context("surface extent overflow")?;
        let permission = self.permission(tile)?;
        let terrain = describe_collision(permission).terrain;
        let mut edges = Vec::new();
        for direction in DIRECTIONS {
            let (dx, dy) = direction.delta();
            let nx = i32::from(tile[0]) + i32::from(dx);
            let ny = i32::from(tile[1]) + i32::from(dy);
            if (0..i32::from(width)).contains(&nx) && (0..i32::from(height)).contains(&ny) {
                let target = [u16::try_from(nx)?, u16::try_from(ny)?];
                let target_permission = self.permission(target)?;
                let target_terrain = describe_collision(target_permission).terrain;
                let mode = if target_terrain == Terrain::Water {
                    PlayerTraversalState::Surf
                } else {
                    PlayerTraversalState::Walk
                };
                let mounting = terrain == Terrain::Land && target_terrain == Terrain::Water;
                if !occupied.contains(&target)
                    && (allow_surf || target_terrain != Terrain::Water)
                    && !is_direction_blocked_leaving(permission, direction)
                    && is_permission_passable(target_permission, direction, mode)
                    && (!mounting || !self.blocked_mounts.contains(&target_permission))
                {
                    edges.push((target, 1));
                }
            }
            // Reuse the native complementary-quadrant, two-tile hop and
            // landing/body checks rather than inventing a second ledge model.
            let mut player = PlayerMovementState::new(TilePosition::new(
                i16::try_from(tile[0])?,
                i16::try_from(tile[1])?,
            ))
            .with_mode(if terrain == Terrain::Water {
                MovementMode::Surf
            } else {
                MovementMode::Normal
            });
            if let LedgeJumpOutcome::Jumped { to, .. } = attempt_ledge_jump_with_occupied_tiles(
                &mut player,
                direction,
                &self.map,
                &self.collision,
                StepOptions::default(),
                bodies,
            ) {
                let target = [u16::try_from(to.x)?, u16::try_from(to.y)?];
                let steps =
                    u32::from(tile[0].abs_diff(target[0])) + u32::from(tile[1].abs_diff(target[1]));
                edges.push((target, steps));
            }
        }
        Ok(edges)
    }

    fn validate_outer(&self, seen: &BTreeSet<[u16; 2]>) -> Result<()> {
        let (width, height) = self
            .map
            .checked_tile_bounds()
            .context("surface extent overflow")?;
        for &[x, y] in seen {
            let permission = self.permission([x, y])?;
            if describe_collision(permission).terrain != Terrain::Water {
                continue;
            }
            for (on_edge, direction) in [
                (x == 0, Direction::Left),
                (x + 1 == width, Direction::Right),
                (y == 0, Direction::Up),
                (y + 1 == height, Direction::Down),
            ] {
                ensure!(
                    !on_edge || is_direction_blocked_leaving(permission, direction),
                    "reachable outer water is unclosed at {x},{y} toward {direction:?}"
                );
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;
    use std::sync::OnceLock;

    use crystal_core::world::collision::{MetatileCollision, TilesetCollision, permissions};
    use crystal_core::world::map::OverworldMapData;

    use super::{
        Surface, opened_cut_blocks, reachable_with_surf, shortest_path_distance,
        validate_outer_boundary,
    };
    use crate::CompiledGamePack;
    use crate::authored_route_design::{CutGate, RouteDesign, Shortcut};

    fn surface(grid: &[Vec<u8>]) -> Surface {
        let height = grid.len();
        let width = grid[0].len();
        assert_eq!(height % 2, 0);
        assert_eq!(width % 2, 0);
        let mut metatiles = Vec::new();
        let mut ids = Vec::new();
        for y in (0..height).step_by(2) {
            for x in (0..width).step_by(2) {
                ids.push(u16::try_from(metatiles.len()).expect("fixture ID"));
                metatiles.push(MetatileCollision {
                    collision: [
                        grid[y][x],
                        grid[y][x + 1],
                        grid[y + 1][x],
                        grid[y + 1][x + 1],
                    ],
                });
            }
        }
        Surface {
            map: OverworldMapData {
                name: "numeric_surface_fixture".into(),
                width: u16::try_from(width / 2).expect("width"),
                height: u16::try_from(height / 2).expect("height"),
                border_block: 0,
                connections: Vec::new(),
                metatile_ids: ids,
            },
            collision: TilesetCollision { metatiles },
            blocked_mounts: Vec::new(),
        }
    }

    #[test]
    fn reachable_water_21_outer_edge_is_rejected() {
        let mut grid = vec![vec![permissions::WALL; 8]; 8];
        grid[2][2] = permissions::FLOOR;
        grid[2][3..8].fill(0x21);
        let surface = surface(&grid);
        let seen = surface
            .reachable([2, 2], &BTreeSet::new())
            .expect("numeric reachability");
        assert!(seen.contains(&[7, 2]));
        assert!(surface.validate_outer(&seen).is_err());
    }

    #[test]
    fn fenced_outer_water_21_drainage_is_not_an_escape() {
        let mut grid = vec![vec![permissions::WALL; 8]; 8];
        grid[2][2] = permissions::FLOOR;
        grid[2][3] = permissions::FLOOR;
        for row in &mut grid {
            row[7] = 0x21;
        }
        let surface = surface(&grid);
        let seen = surface
            .reachable([2, 2], &BTreeSet::new())
            .expect("fenced reachability");
        assert!(!seen.contains(&[7, 2]));
        surface
            .validate_outer(&seen)
            .expect("genuinely inaccessible drainage");
    }

    #[test]
    fn surf_mount_honors_independent_departure_and_entry_masks() {
        for (source, target) in [
            (permissions::RIGHT_WALL, 0x21),
            (permissions::FLOOR, permissions::LEFT_BUOY),
        ] {
            let mut grid = vec![vec![permissions::WALL; 8]; 8];
            grid[2][2] = source;
            grid[2][3] = target;
            assert!(
                !surface(&grid)
                    .reachable([2, 2], &BTreeSet::new())
                    .expect("masked mount")
                    .contains(&[3, 2])
            );
        }
        let mut grid = vec![vec![permissions::WALL; 8]; 8];
        grid[2][2] = permissions::FLOOR;
        grid[2][3] = 0x21;
        let mut surface = surface(&grid);
        surface.blocked_mounts.push(0x21);
        assert!(
            !surface
                .reachable([2, 2], &BTreeSet::new())
                .expect("catalog blocked mount")
                .contains(&[3, 2])
        );
    }

    #[test]
    fn outward_directional_buoy_is_a_real_boundary() {
        let mut grid = vec![vec![permissions::WALL; 8]; 8];
        grid[2][2] = permissions::FLOOR;
        grid[2][3..7].fill(0x21);
        grid[2][7] = permissions::RIGHT_BUOY;
        let surface = surface(&grid);
        let seen = surface
            .reachable([2, 2], &BTreeSet::new())
            .expect("buoy approach");
        assert!(seen.contains(&[7, 2]));
        surface
            .validate_outer(&seen)
            .expect("native departure mask closes outer edge");
    }

    #[test]
    fn lake_crosses_internal_chunk_seam_and_dismounts_on_far_bank() {
        let mut grid = vec![vec![permissions::WALL; 68]; 8];
        grid[2][2] = permissions::FLOOR;
        grid[2][3..66].fill(0x21);
        grid[2][66] = permissions::FLOOR;
        let surface = surface(&grid);
        let seen = surface
            .reachable([2, 2], &BTreeSet::new())
            .expect("whole-route lake seam");
        assert!(seen.contains(&[63, 2]) && seen.contains(&[64, 2]) && seen.contains(&[66, 2]));
        surface
            .validate_outer(&seen)
            .expect("internal seam is not an outside boundary");
    }

    #[test]
    fn occupied_launch_and_masked_dismount_are_not_free_connections() {
        let mut grid = vec![vec![permissions::WALL; 8]; 8];
        grid[2][2] = permissions::FLOOR;
        grid[2][3] = 0x21;
        grid[2][4] = permissions::LEFT_WALL;
        let surface = surface(&grid);
        let seen = surface
            .reachable([2, 2], &BTreeSet::new())
            .expect("legal launch");
        assert!(seen.contains(&[3, 2]));
        assert!(!seen.contains(&[4, 2]));
        let occupied = BTreeSet::from([[3, 2]]);
        let blocked = surface
            .reachable([2, 2], &occupied)
            .expect("occupied launch");
        assert!(!blocked.contains(&[3, 2]));
    }

    #[test]
    fn native_ledge_primitive_reaches_lower_bank_without_reverse_ascent() {
        let mut grid = vec![vec![permissions::WALL; 8]; 8];
        grid[2][2] = permissions::FLOOR;
        grid[3][2] = permissions::HOP_DOWN;
        grid[5][2] = permissions::FLOOR;
        let surface = surface(&grid);
        assert!(
            surface
                .reachable([2, 2], &BTreeSet::new())
                .expect("native hop")
                .contains(&[2, 5])
        );
        assert!(
            !surface
                .reachable([2, 5], &BTreeSet::new())
                .expect("lower bank")
                .contains(&[2, 2])
        );
        assert!(
            !surface
                .reachable([2, 2], &BTreeSet::from([[2, 5]]))
                .expect("occupied hop landing")
                .contains(&[2, 5])
        );
    }

    #[test]
    fn distances_share_native_surf_edges_and_count_hop_tile_displacement() {
        let mut grid = vec![vec![permissions::WALL; 8]; 8];
        grid[2][2] = permissions::FLOOR;
        grid[2][3] = 0x21;
        grid[2][4] = permissions::FLOOR;
        let lake = surface(&grid);
        assert!(
            !lake
                .shortest_paths([2, 2], &BTreeSet::new(), false, Some([4, 2]))
                .expect("walking only")
                .contains_key(&[4, 2])
        );
        assert_eq!(
            lake.shortest_paths([2, 2], &BTreeSet::new(), true, Some([4, 2]))
                .expect("Surf distance")
                .get(&[4, 2]),
            Some(&2)
        );
        grid[2][3] = permissions::WALL;
        grid[2][4] = permissions::WALL;
        grid[3][2] = permissions::HOP_DOWN;
        grid[5][2] = permissions::FLOOR;
        let ledge = surface(&grid);
        assert_eq!(
            ledge
                .shortest_paths([2, 2], &BTreeSet::new(), false, Some([2, 5]))
                .expect("hop distance")
                .get(&[2, 5]),
            Some(&3)
        );
        assert!(
            !ledge
                .shortest_paths([2, 5], &BTreeSet::new(), false, Some([2, 2]))
                .expect("hop reverse")
                .contains_key(&[2, 2])
        );
    }

    #[test]
    fn current_cut_shortcuts_have_measured_walking_and_surf_savings() {
        let root =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../planning/minnesota/routes");
        for (route, from, to, before, after) in [
            ("MN_R14", [71, 21], [71, 19], 34, 2),
            ("MN_R17", [36, 67], [36, 73], 66, 6),
        ] {
            let dossier = std::fs::read_to_string(root.join(format!("{route}.json")))
                .expect("current dossier");
            let design: RouteDesign =
                serde_json::from_str(&dossier).expect("typed distance fixture");
            let layout_text = std::fs::read_to_string(root.join(format!("{route}.layout.json")))
                .expect("current strokes");
            let layout = crate::authored_route_layout::Layout::parse(&layout_text)
                .expect("native layout parser");
            let blocks = layout.compile().expect("native explicit strokes");
            let pack = if route == "MN_R14" {
                crate::headwaters_art::build_headwaters_art_pack(source())
                    .expect("existing native headwaters art")
            } else {
                let mut pack =
                    crate::build_gunflint_art_pack(source()).expect("existing native forest art");
                // Numeric-only fixture for the two new rail collisions. Source
                // recipes are exact Kanto77 and Johto75; this does not claim a
                // published Border Pines graphics extension or controller run.
                let mut definition = pack.data.tilesets["mn_gunflint"].clone();
                definition.collision.insert(
                    "3a".into(),
                    pack.data.tilesets["kanto"].collision["77"].clone(),
                );
                definition.collision.insert(
                    "3b".into(),
                    pack.data.tilesets["johto"].collision["75"].clone(),
                );
                pack.data
                    .tilesets
                    .insert("mn_border_pines".into(), definition);
                let cut = pack.data.field_moves.cut.replacements["mn_gunflint"].clone();
                pack.data
                    .field_moves
                    .cut
                    .replacements
                    .insert("mn_border_pines".into(), cut);
                pack
            };
            let opened = opened_cut_blocks(&pack, &design, &blocks, &layout.compiled_tileset)
                .expect("actual native Cut mutation");
            for surf in [false, true] {
                assert_eq!(
                    shortest_path_distance(
                        &pack,
                        &design,
                        &blocks,
                        &layout.compiled_tileset,
                        &BTreeSet::new(),
                        from,
                        to,
                        surf
                    )
                    .expect("closed native distance"),
                    Some(before),
                    "{route}, Surf={surf}"
                );
                assert_eq!(
                    shortest_path_distance(
                        &pack,
                        &design,
                        &opened,
                        &layout.compiled_tileset,
                        &BTreeSet::new(),
                        from,
                        to,
                        surf
                    )
                    .expect("opened native distance"),
                    Some(after),
                    "{route}, Surf={surf}"
                );
            }
        }
    }

    fn source() -> &'static CompiledGamePack {
        static SOURCE: OnceLock<CompiledGamePack> = OnceLock::new();
        SOURCE.get_or_init(|| {
            crate::read_verified_compiled_game_pack(
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../content-packs/text-tui.crystalpack")
                    .canonicalize()
                    .expect("canonical supplied pack"),
            )
            .expect("supplied native source")
        })
    }

    #[test]
    fn boundary_opens_actual_native_cut_replacement_and_legacy_shortcut() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut design = RouteDesign::parse(
            &std::fs::read_to_string(root.join("../../planning/minnesota/routes/MN_R01.json"))
                .expect("existing typed route"),
        )
        .expect("route fixture");
        design.dimensions_metatiles = [4, 4];
        design.endpoints[0].gate_metatile = [1, 2];
        design.items.clear();
        design.cut_gates.clear();
        design.shortcut = None;
        let mut blocks = vec![5; 16];
        blocks[2 * 4 + 1] = 1;
        blocks[4 + 1] = 0x5b;
        blocks[1] = 0x35;
        let closed = reachable_with_surf(source(), &design, &blocks, "johto", &BTreeSet::new())
            .expect("closed native tree");
        assert!(!closed.contains(&[3, 0]));
        design.cut_gates.push(CutGate {
            id: "fixture_gate".into(),
            tile: [3, 2],
            approach: [3, 3],
            replacement_block: "3c".into(),
            targets: vec![[3, 1]],
            gated_actor_ids: Vec::new(),
        });
        let error = validate_outer_boundary(source(), &design, &blocks, "johto", &BTreeSet::new())
            .expect_err("actual native Cut must expose the water escape");
        assert!(
            error.to_string().contains("reachable outer water"),
            "wrong failure: {error}"
        );
        let occupied = BTreeSet::from([[3, 3]]);
        let error = validate_outer_boundary(source(), &design, &blocks, "johto", &occupied)
            .expect_err("a movable NPC cannot make an unsafe boundary valid");
        assert!(
            error.to_string().contains("reachable outer water"),
            "wrong failure: {error}"
        );
        design.cut_gates.clear();
        design.shortcut = Some(Shortcut {
            tile: [3, 2],
            approach: [3, 3],
            compare_from: None,
            compare_to: None,
            minimum_saved_steps: None,
            useful_target_actor_ids: Vec::new(),
        });
        let error = validate_outer_boundary(source(), &design, &blocks, "johto", &BTreeSet::new())
            .expect_err("actual native Cut must expose the water escape");
        assert!(
            error.to_string().contains("reachable outer water"),
            "wrong failure: {error}"
        );
    }
}
