//! Read-only direction evidence from the same authored collision catalog
//! used by production movement. No height or permission feeds back to gameplay.
use super::*;
use crystal_render_api::VisualJumpLedge;

#[cfg(feature="voxel-view")]
pub(super) fn datum(sources:&[ResolvedRenderSource<'_>],runtime:&CrystalRuntime,
    edges:&[VisualJumpLedge])->anyhow::Result<Option<std::sync::Arc<crystal_render_api::VisualTerraceDatum>>> {
    let Some(primary)=sources.first() else{return Ok(None);};
    let mut regions=Vec::new();
    // The active map wins; connected rectangles use the renderer's reverse
    // lookup order (nearest connections precede second-ring continuations).
    for source in sources.iter().take(1).chain(sources.iter().skip(1).rev()) {
        let mut cells=Vec::new();
        let atlas=&source.tileset.tileset_id;
        let collision=runtime.data().tileset_collision(atlas)?;
        let layout=runtime.runtime_file(&format!("data/tilesets/{atlas}_metatiles.bin"))
            .ok_or_else(||anyhow::anyhow!("missing terrace source layout {atlas}"))?;
        for y in 0..source.height {for x in 0..source.width {
            let block=source.map.blocks[(y as usize/4)*source.map.attributes.width as usize+x as usize/4];
            let tile=*layout.get(block as usize*16+(y as usize%4)*4+x as usize%4)
                .ok_or_else(||anyhow::anyhow!("missing terrace source metatile {block}"))?;
            let permission=collision.metatiles[block as usize].collision[(y as usize%4/2)*2+x as usize%4/2];
            let land=crystal_core::world::collision::describe_collision(permission).terrain
                ==crystal_core::world::collision::Terrain::Land
                || [Direction::Up,Direction::Down,Direction::Left,Direction::Right].into_iter()
                    .any(|d|crystal_core::world::collision::allows_ledge_direction(permission,d));
            cells.push((IVec2::new(source.origin_x+x,source.origin_y+y),crystal_render_api::VisualTileSource {
                tileset_id:atlas.as_str().into(),metatile_id:block,subtile_column:(x%4)as u8,
                subtile_row:(y%4)as u8,tile_index:u16::from(tile),
            },land));
        }}
        regions.push(crystal_voxel_view::TerraceSourceRegion {
            map:source.map.map_name.to_string(),cells,
            edges:from_map(source.map.attributes.width,&source.map.blocks,&collision,
                IVec2::new(source.origin_x,source.origin_y))?.to_vec(),
        });
    }
    if crystal_voxel_view::outdoor_water_map(&primary.map.map_name) {
        // Include adjacent source border water at the complete perimeter so
        // a clipped camera cannot leave that water beneath a raised dry terrace.
        // Native maps always retain ownership, including connected-map overlaps.
        let mut perimeter=std::collections::BTreeSet::new();
        for source in sources {
            for y in source.origin_y-1..=source.origin_y+source.height {
                for x in source.origin_x-1..=source.origin_x+source.width {
                    if x!=source.origin_x-1 && x!=source.origin_x+source.width
                        && y!=source.origin_y-1 && y!=source.origin_y+source.height {continue;}
                    if sources.iter().any(|s|(s.origin_x..s.origin_x+s.width).contains(&x)
                        && (s.origin_y..s.origin_y+s.height).contains(&y)) {continue;}
                    perimeter.insert((y,x));
                }
            }
        }
        let mut border_water=Vec::new();
        for (y,x) in perimeter {
            // A synthetic perimeter must use its adjacent native map's border,
            // otherwise crossing into a watery route lowers the neighbor's
            // existing cliff corners during shoreline fitting.
            let source = sources.iter().min_by_key(|s| {
                let dx=(s.origin_x-x).max(0)+(x-(s.origin_x+s.width-1)).max(0);
                let dy=(s.origin_y-y).max(0)+(y-(s.origin_y+s.height-1)).max(0);
                (dx+dy,s.map.map_name.as_str())
            }).expect("primary source exists");
            let atlas=&source.tileset.tileset_id;
            let layout=runtime.runtime_file(&format!("data/tilesets/{atlas}_metatiles.bin"))
                .ok_or_else(||anyhow::anyhow!("missing border terrace source layout {atlas}"))?;
            let (local_x,local_y)=(x-source.origin_x,y-source.origin_y);
            let block=exterior_border_block(&source.map.map_name,&source.map.attributes.tileset_name,
                source.map.attributes.border_block,source.width,local_x,local_y);
            let (column,row)=(local_x.rem_euclid(4) as usize,local_y.rem_euclid(4) as usize);
            let tile=*layout.get(block as usize*16+row*4+column)
                .ok_or_else(||anyhow::anyhow!("missing border terrace source metatile {block}"))?;
            let cell=crystal_render_api::VisualTileSource {
                tileset_id:atlas.as_str().into(),metatile_id:block,subtile_column:column as u8,
                subtile_row:row as u8,tile_index:u16::from(tile),
            };
            if crystal_voxel_view::shape_for_source(&cell)==crystal_voxel_view::CellShape::Water {
                border_water.push((IVec2::new(x,y),cell,false));
            }
        }
        if !border_water.is_empty() {
            regions.push(crystal_voxel_view::TerraceSourceRegion {
                map:primary.map.map_name.to_string(),cells:border_water,edges:Vec::new(),
            });
        }
    }
    Ok(crystal_voxel_view::connected_source_terrace_datum(&primary.map.map_name,&regions,edges))
}

pub(super) fn collect(
    sources: &[ResolvedRenderSource<'_>],
    data: &crystal_assets::GameDataSet,
) -> anyhow::Result<std::sync::Arc<[VisualJumpLedge]>> {
    let mut edges = Vec::new();
    for source in sources {
        let collision = data.tileset_collision(&source.tileset.tileset_id)?;
        append_map(
            &mut edges,
            source.map.attributes.width,
            &source.map.blocks,
            &collision,
            IVec2::new(source.origin_x, source.origin_y),
        )?;
    }
    Ok(merge(edges).into())
}

pub(super) fn from_map(
    width: u16,
    blocks: &[u16],
    collision: &crystal_core::world::collision::TilesetCollision,
    origin: IVec2,
) -> anyhow::Result<std::sync::Arc<[VisualJumpLedge]>> {
    let mut edges = Vec::new();
    append_map(&mut edges, width, blocks, collision, origin)?;
    Ok(merge(edges).into())
}

fn append_map(
    edges: &mut Vec<VisualJumpLedge>,
    width: u16,
    blocks: &[u16],
    collision: &crystal_core::world::collision::TilesetCollision,
    origin: IVec2,
) -> anyhow::Result<()> {
    anyhow::ensure!(width > 0, "ledge source map width must be positive");
    for (i, &block) in blocks.iter().enumerate() {
        let art = collision
            .metatiles
            .get(usize::from(block))
            .ok_or_else(|| anyhow::anyhow!("missing ledge source collision for block {block}"))?;
        let block_origin = origin
            + IVec2::new(
                (i % usize::from(width)) as i32 * 4,
                (i / usize::from(width)) as i32 * 4,
            );
        append_block(edges, block_origin, art.collision);
    }
    Ok(())
}

fn merge(edges: Vec<VisualJumpLedge>) -> Vec<VisualJumpLedge> {
    // Adjacent collision quadrants form one visual edge. Merge before
    // presentation so a four-cell course cannot count as two elevation drops.
    let mut runs = std::collections::BTreeMap::<(i32, i32, i32), Vec<(i32, i32)>>::new();
    for edge in edges {
        let vertical = edge.descent.x != 0;
        let plane = if vertical { edge.start.x } else { edge.start.y };
        let span = if vertical {
            (edge.start.y, edge.end.y)
        } else {
            (edge.start.x, edge.end.x)
        };
        runs.entry((edge.descent.x, edge.descent.y, plane))
            .or_default()
            .push(span);
    }
    let mut result = Vec::new();
    for ((dx, dy, plane), mut spans) in runs {
        spans.sort_unstable();
        let mut merged = Vec::<(i32, i32)>::new();
        for (a, b) in spans {
            if let Some(last) = merged.last_mut().filter(|last| a <= last.1) {
                last.1 = last.1.max(b);
            } else {
                merged.push((a, b));
            }
        }
        for (a, b) in merged {
            result.push(VisualJumpLedge {
                start: if dx != 0 {
                    IVec2::new(plane, a)
                } else {
                    IVec2::new(a, plane)
                },
                end: if dx != 0 {
                    IVec2::new(plane, b)
                } else {
                    IVec2::new(b, plane)
                },
                descent: IVec2::new(dx, dy),
            });
        }
    }
    result
}

fn append_block(edges: &mut Vec<VisualJumpLedge>, origin: IVec2, collision: [u8; 4]) {
    use crystal_core::world::collision::allows_ledge_direction;
    for (q, permission) in collision.into_iter().enumerate() {
        let corner = [
            Direction::Down,
            Direction::Up,
            Direction::Right,
            Direction::Left,
        ]
        .into_iter()
        .filter(|&d| allows_ledge_direction(permission, d))
        .count()
            > 1;
        for (direction, descent) in [
            (Direction::Down, IVec2::Y),
            (Direction::Up, IVec2::NEG_Y),
            (Direction::Right, IVec2::X),
            (Direction::Left, IVec2::NEG_X),
        ] {
            if !allows_ledge_direction(permission, direction) {
                continue;
            }
            let (a, b) = if descent.y != 0 {
                let y = if descent.y > 0 { 4 } else { 0 };
                let x = if corner { 0 } else { (q % 2) as i32 * 2 };
                (
                    IVec2::new(x, y),
                    IVec2::new(x + if corner { 4 } else { 2 }, y),
                )
            } else {
                let x = if descent.x > 0 { 4 } else { 0 };
                let y = if corner { 0 } else { (q / 2) as i32 * 2 };
                (
                    IVec2::new(x, y),
                    IVec2::new(x, y + if corner { 4 } else { 2 }),
                )
            };
            edges.push(VisualJumpLedge {
                start: origin + a,
                end: origin + b,
                descent,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crystal_core::world::collision::permissions::*;
    #[test]
    fn permission_directions_point_from_jumpable_high_side_to_blocked_low_side() {
        for (permission, descent) in [
            (HOP_DOWN, IVec2::Y),
            (HOP_UP, IVec2::NEG_Y),
            (HOP_LEFT, IVec2::NEG_X),
            (HOP_RIGHT, IVec2::X),
        ] {
            let mut edges = Vec::new();
            append_block(
                &mut edges,
                IVec2::new(20, 40),
                [permission, WALL, WALL, WALL],
            );
            assert_eq!(edges.len(), 1);
            assert_eq!(edges[0].descent, descent);
        }
        let mut edges = Vec::new();
        append_block(&mut edges, IVec2::ZERO, [FLOOR, WALL, WALL, WALL]);
        assert!(
            edges.is_empty(),
            "a wall or ordinary floor is not a jump direction"
        );
        append_block(&mut edges, IVec2::ZERO, [HOP_DOWN_LEFT, WALL, WALL, WALL]);
        assert_eq!(edges.len(), 2);
        assert!(edges.iter().any(|e| e.descent == IVec2::Y));
        assert!(edges.iter().any(|e| e.descent == IVec2::NEG_X));
    }
}
