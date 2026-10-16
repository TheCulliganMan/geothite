//! Continue the room's existing rear architecture around its actual map bounds.
//! All geometry is presentation-only; source floors and support remain intact.
use super::*;
use bevy::prelude::IVec2;
#[path = "tin_room_bounds.rs"]
mod tin_room_bounds;
#[cfg(test)]
use bevy::prelude::UVec2;

fn is_wall(kind: ModelKind) -> bool {
    matches!(
        kind,
        ModelKind::WallDomestic
            | ModelKind::WallTraditional
            | ModelKind::WallClinical
            | ModelKind::WallAcoustic
            | ModelKind::WallPanel
            | ModelKind::ArcadeWall
            | ModelKind::WindowWall
    )
}

fn plain_panel(p: &Placement, cells: &[&VisualTile], g: &GridGeometry) -> bool {
    let source = &cells[p.row * g.width + p.column].source;
    p.indices(g.width)
        .all(|i| cells[i].source.tile_index == source.tile_index)
}

fn ungrounded_wall_donor(map: &str, cells: &[&VisualTile], g: &GridGeometry) -> Option<Placement> {
    let mut drawings = INTERIOR_DRAWINGS
        .iter()
        .filter(|d| is_wall(d.kind))
        .collect::<Vec<_>>();
    drawings.sort_by_key(|d| {
        (
            d.kind == ModelKind::WindowWall,
            !d.tiles.iter().all(|t| *t == d.tiles[0]),
        )
    });
    for drawing in drawings {
        if drawing.tileset == "underground" && map != "VermilionPortPassage" {
            continue;
        }
        if map == "CeladonGameCornerPrizeRoom" && drawing.tileset == "game_corner" {
            continue;
        }
        if reviewed_native_room(map)
            && drawing.tileset == "players_house"
            && drawing.tiles.iter().all(|t| *t == 0x44)
        {
            continue;
        }
        let [w, h] = drawing.size;
        if w > g.width || h > g.height {
            continue;
        }
        for row in 0..=g.height - h {
            for column in 0..=g.width - w {
                if !(0..h).all(|dy| {
                    (0..w).all(|dx| {
                        let s = &cells[(row + dy) * g.width + column + dx].source;
                        s.tileset_id.as_ref() == drawing.tileset
                            && s.metatile_id == drawing.block
                            && s.subtile_column as usize == drawing.origin[0] as usize + dx
                            && s.subtile_row as usize == drawing.origin[1] as usize + dy
                            && s.tile_index == drawing.tiles[dy * w + dx]
                    })
                }) {
                    continue;
                }
                let (depth_pixels, height_pixels) = proportions(drawing.kind, w, h);
                return Some(Placement {
                    kind: drawing.kind,
                    column,
                    row,
                    width: w,
                    height: h,
                    // This temporary floor is discarded by above_floor. Only
                    // complete authenticated wall geometry reaches the room shell.
                    ground: row * g.width + column,
                    depth_pixels,
                    height_pixels,
                    base_pixels: 0.,
                    footing_pixels: None,
                });
            }
        }
    }
    None
}

fn above_floor(source: &SurfaceMeshData) -> SurfaceMeshData {
    let indices = source
        .indices
        .chunks_exact(3)
        .filter(|triangle| {
            triangle
                .iter()
                .any(|&i| source.positions[i as usize][1] > 0.01)
        })
        .flatten()
        .copied();
    select_surface_indices(
        source,
        &vec![false; source.positions.len()],
        &mut vec![u32::MAX; source.positions.len()],
        indices,
    )
}

/// Some rear courses are entirely occupied by equipment or built-in shelving.
/// Their returns use the rear drawing's field paint, never repeated furniture.
fn rear_paint_template(
    out: &mut TerrainMeshData,
    existing: &TerrainMeshData,
    cells: &[&VisualTile],
    g: &GridGeometry,
    frame: &VisualWorldFrame,
    images: Option<&TerrainImageSamples>,
) -> Option<usize> {
    let size = frame.source_map_size_core_tiles?;
    let row = (0..g.height).find(|&row| frame.grid_origin.y + row as i32 >= 0)?;
    let mut paint = std::collections::BTreeMap::<[u8; 4], (usize, usize, usize)>::new();
    for (i, tile) in cells.iter().enumerate().filter(|(i, _)| i / g.width == row) {
        let sx = frame.grid_origin.x + (i % g.width) as i32;
        if sx < 0 || sx >= size.x as i32 * 2 || tile.source.metatile_id == u16::MAX {
            continue;
        }
        if let Some(TileImageSample::Rgba(pixels)) =
            images.and_then(|images| images.pixels.get(&tile.texture.id()))
        {
            for (pixel, rgba) in pixels
                .chunks_exact(4)
                .enumerate()
                .filter(|(_, p)| p[3] == 255)
            {
                let entry = paint
                    .entry(rgba.try_into().unwrap())
                    .or_insert((0, i, pixel));
                entry.0 += 1;
            }
        } else {
            // Geometry-only callers do not provide pixel samples.
            paint.entry([0, 0, 0, 255]).or_insert((1, i, 0));
        }
    }
    let (_, (_, donor, pixel)) = paint.into_iter().max_by_key(|(_, v)| v.0)?;
    let uv = g.uv(donor % g.width, donor / g.width);
    let sample = [
        uv.0 + (uv.1 - uv.0) * ((pixel % 8) as f32 + 0.5) / 8.,
        uv.2 + (uv.3 - uv.2) * ((pixel / 8) as f32 + 0.5) / 8.,
    ];
    let north = g.origin_z - frame.grid_origin.y as f32 * g.tile_height;
    let west = g.origin_x - frame.grid_origin.x as f32 * g.tile_width;
    let height = existing
        .solid
        .positions
        .iter()
        .chain(&existing.textured.positions)
        .filter(|p| {
            p[0] >= west
                && p[0] <= west + size.x as f32 * 2. * g.tile_width
                && p[2] >= north
                && p[2] <= north + 4. * g.tile_height
        })
        .map(|p| p[1])
        .fold(0., f32::max);
    let height = if height > g.tile_height {
        height
    } else {
        4. * g.tile_height
    };
    let front = g.origin_z + (row + 2) as f32 * g.tile_height;
    architecture_box(
        &mut out.textured,
        [
            west,
            west + 2. * g.tile_width,
            0.,
            height,
            front - 3. * g.tile_height / 8.,
            front,
        ],
        [1.; 3],
    );
    out.textured.uvs.fill(sample);
    Some(row)
}

/// Legacy rooms retain source-band walls instead of a modeled panel. Continue
/// those exact bands, including their atlas paint, over a closed backing.
fn source_band_template(
    mesh: &mut TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    origin: IVec2,
) -> Option<usize> {
    for row in 0..g.height {
        for column in 0..g.width.saturating_sub(1) {
            let sx = origin.x + column as i32;
            let sy = origin.y + row as i32;
            if sx < 0 || sy < 0 || sy > 3 {
                continue;
            }
            if crate::mart::department_store_wall_shape(map, &cells[row * g.width + column].source)
                .is_none()
            {
                continue;
            }
            let CellShape::FacadeBand {
                plane_subtile_row,
                band_from_top: 0,
                band_count,
                solid: SolidKind::FlatCard,
                ..
            } = shape_for_source_on_map(map, &cells[row * g.width + column].source)
            else {
                continue;
            };
            if band_count < 2 || row + band_count as usize > g.height {
                continue;
            }
            if !(0..band_count as usize).all(|dy|(0..2).all(|dx| matches!(
            shape_for_source_on_map(map,&cells[(row+dy)*g.width+column+dx].source),
            CellShape::FacadeBand{plane_subtile_row:p,band_from_top:b,band_count:c,solid:SolidKind::FlatCard,..}
                if p==plane_subtile_row&&b as usize==dy&&c==band_count))) {continue;}
            let west = g.origin_x + column as f32 * g.tile_width;
            let east = west + 2. * g.tile_width;
            let front = g.origin_z
                + (row as f32 - cells[row * g.width + column].source.subtile_row as f32
                    + plane_subtile_row as f32)
                    * g.tile_height;
            let height = band_count as f32 * g.tile_height;
            let uv = g.uv(column, row);
            let paint = [
                uv.0 + (uv.1 - uv.0) * 0.5 / 8.,
                uv.2 + (uv.3 - uv.2) * 0.5 / 8.,
            ];
            let first = mesh.textured.positions.len();
            architecture_box(
                &mut mesh.textured,
                [
                    west,
                    east,
                    0.,
                    height,
                    front - 3. * g.tile_height / 8.,
                    front,
                ],
                [1.; 3],
            );
            mesh.textured.uvs[first..].fill(paint);
            for dy in 0..band_count as usize {
                for dx in 0..2 {
                    let uv = g.uv(column + dx, row + dy);
                    let w = west + dx as f32 * g.tile_width;
                    let e = w + g.tile_width;
                    let top = height - dy as f32 * g.tile_height;
                    let bottom = top - g.tile_height;
                    append_quad(
                        &mut mesh.textured,
                        [
                            [e, bottom, front + 0.001],
                            [e, top, front + 0.001],
                            [w, top, front + 0.001],
                            [w, bottom, front + 0.001],
                        ],
                        Vec3::Z.to_array(),
                        [[uv.1, uv.3], [uv.1, uv.2], [uv.0, uv.2], [uv.0, uv.3]],
                        TEXTURED_SHADE,
                    );
                }
            }
            return Some(row);
        }
    }
    None
}

#[derive(Clone, Copy, Debug)]
enum Edge {
    West,
    East,
    South,
}

fn copy_section(
    out: &mut SurfaceMeshData,
    source: &SurfaceMeshData,
    b: [f32; 4],
    edge: Edge,
    at: f32,
    start: f32,
    end: f32,
) {
    if source.indices.is_empty() {
        return;
    }
    let first = out.positions.len();
    let stretch = (end - start) / (b[1] - b[0]);
    let turn = |x: f32, y: f32, z: f32| match edge {
        Edge::West => [z, y, -x],
        Edge::East => [-z, y, x],
        Edge::South => [-x, y, -z],
    };
    out.positions.extend(source.positions.iter().map(|p| {
        let mut v = turn((p[0] - b[0]) * stretch, p[1], p[2] - b[3]);
        match edge {
            Edge::West => {
                v[0] += at;
                v[2] += end;
            }
            Edge::East => {
                v[0] += at;
                v[2] += start;
            }
            Edge::South => {
                v[0] += end;
                v[2] += at;
            }
        }
        v
    }));
    out.normals.extend(source.normals.iter().map(|n| {
        Vec3::from_array(turn(n[0] / stretch, n[1], n[2]))
            .normalize()
            .to_array()
    }));
    out.uvs.extend_from_slice(&source.uvs);
    out.colors.extend_from_slice(&source.colors);
    out.indices
        .extend(source.indices.iter().map(|i| *i + first as u32));
    out.cutaway_ranges.push(first..out.positions.len());
}

pub(in crate::mesh) fn append(
    mesh: &mut TerrainMeshData,
    frame: &VisualWorldFrame,
    cells: &[&VisualTile],
    g: &GridGeometry,
    placements: &[Placement],
    dungeon_placements: &[super::super::modeled_dungeons::Placement],
    images: Option<&TerrainImageSamples>,
) {
    let Some(size) = frame.source_map_size_core_tiles else {
        return;
    };
    let Some(source) = cells.first() else {
        return;
    };
    if !crate::interior::has_room_perimeter(&frame.map_id, &source.source.tileset_id) {
        return;
    }
    // Resolve the source drawing through the same production wall builder.
    // Prefer plain panels; a glazed room can continue its window course.
    // Never repeat signs, cabinets or a whole furniture course.
    let donor = placements
        .iter()
        .filter(|p| is_wall(p.kind) && p.base_pixels == 0. && p.depth_pixels <= 16.)
        .min_by_key(|p| {
            (
                p.kind == ModelKind::WindowWall,
                p.row,
                !plain_panel(p, cells, g),
                p.column,
            )
        });
    let mut template = TerrainMeshData::default();
    let donor_row = if let Some(row) = traditional_wood_wall::perimeter_template(
        &mut template.textured,
        &frame.map_id,
        cells,
        g,
        placements,
    ) {
        row
    } else if let Some(donor) = donor {
        if !super::append(
            &mut template,
            &frame.map_id,
            cells,
            g,
            donor,
            &mut vec![false; cells.len()],
            frame.grid_origin.to_array(),
        ) {
            return;
        }
        donor.row
    } else if let Some(donor) = dungeon_placements
        .iter()
        .filter(|p| p.room_wall())
        .min_by_key(|p| p.source_row())
    {
        donor.append_room_wall(&mut template, g, cells, frame.grid_origin.to_array());
        donor.source_row()
    } else if let Some(row) = super::super::department_store::perimeter_template(
        &mut template,
        &frame.map_id,
        cells,
        g,
        frame.grid_origin.to_array(),
        None,
    ) {
        row
    } else if let Some(donor) = ungrounded_wall_donor(&frame.map_id, cells, g) {
        if !super::append(
            &mut template,
            &frame.map_id,
            cells,
            g,
            &donor,
            &mut vec![false; cells.len()],
            frame.grid_origin.to_array(),
        ) {
            return;
        }
        donor.row
    } else if let Some(row) =
        source_band_template(&mut template, &frame.map_id, cells, g, frame.grid_origin)
    {
        row
    } else if let Some(row) = rear_paint_template(&mut template, mesh, cells, g, frame, images) {
        row
    } else {
        return;
    };
    let mut solid = above_floor(&template.solid);
    let mut textured = above_floor(&template.textured);
    let mut b = [
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::INFINITY,
        f32::NEG_INFINITY,
    ];
    for p in solid.positions.iter().chain(&textured.positions) {
        b[0] = b[0].min(p[0]);
        b[1] = b[1].max(p[0]);
        b[2] = b[2].min(p[2]);
        b[3] = b[3].max(p[2]);
    }
    if b.iter().any(|v| !v.is_finite()) || b[1] <= b[0] || b[3] <= b[2] {
        return;
    }
    // Source-band rooms can fold a 16px rear wall while the shared panel kit
    // is 32px tall. Match the actual built rear course at this donor's seam.
    let rear_height = mesh
        .solid
        .positions
        .iter()
        .chain(&mesh.textured.positions)
        .filter(|p| p[0] >= b[0] && p[0] <= b[1] && (p[2] - b[3]).abs() <= g.tile_height * 0.25)
        .map(|p| p[1])
        .fold(0., f32::max);
    let template_height = solid
        .positions
        .iter()
        .chain(&textured.positions)
        .map(|p| p[1])
        .fold(0., f32::max);
    if rear_height >= g.tile_height && template_height > 0. {
        let scale = rear_height / template_height;
        for surface in [&mut solid, &mut textured] {
            for p in &mut surface.positions {
                p[1] *= scale;
            }
            for n in &mut surface.normals {
                *n = Vec3::new(n[0], n[1] / scale, n[2]).normalize().to_array();
            }
        }
    }
    let width = size.x as i32 * 2;
    let height = size.y as i32 * 2;
    let map_west = g.origin_x - frame.grid_origin.x as f32 * g.tile_width;
    let inset = tin_room_bounds::insets(frame, cells, g, width, height);
    let west = map_west + inset[0] as f32 * g.tile_width;
    let east = map_west + (width - inset[1]) as f32 * g.tile_width;
    let north = g.origin_z - frame.grid_origin.y as f32 * g.tile_height;
    let south = north + (height - inset[2]) as f32 * g.tile_height;
    let thickness = b[3] - b[2];
    let rear = north + b[2] - (g.origin_z + donor_row as f32 * g.tile_height);
    // Actual room edges, clipped to this builder's terrain region. A cropped
    // viewport is never a room boundary. Corners overlap outside the floor.
    for (edge, at, from, to, step) in [
        (Edge::West, west, rear.max(north), south, g.tile_height * 2.),
        (Edge::East, east, rear.max(north), south, g.tile_height * 2.),
        (
            Edge::South,
            south,
            west - thickness,
            east + thickness,
            g.tile_width * 2.,
        ),
    ] {
        let (grid_from, grid_to, visible) = match edge {
            Edge::West => (
                g.origin_z,
                g.origin_z + g.height as f32 * g.tile_height,
                frame.grid_origin.x <= inset[0] && frame.grid_origin.x + g.width as i32 >= inset[0],
            ),
            Edge::East => (
                g.origin_z,
                g.origin_z + g.height as f32 * g.tile_height,
                frame.grid_origin.x <= width - inset[1] && frame.grid_origin.x + g.width as i32 >= width - inset[1],
            ),
            Edge::South => (
                g.origin_x,
                g.origin_x + g.width as f32 * g.tile_width,
                frame.grid_origin.y <= height - inset[2] && frame.grid_origin.y + g.height as i32 >= height - inset[2],
            ),
        };
        if !visible {
            continue;
        }
        let mut gaps = frame
            .boundary_warps
            .iter()
            .filter_map(|warp| {
                if warp.x < 0 || warp.y < 0 || warp.x >= size.x as i32 || warp.y >= size.y as i32 {
                    return None;
                }
                let (on_edge, offset, unit) = match edge {
                    Edge::West => (warp.x == inset[0] / 2, warp.y, north),
                    Edge::East => (warp.x == (width - inset[1]) / 2 - 1, warp.y, north),
                    Edge::South => (warp.y >= (height - inset[2]) / 2 - 2, warp.x, map_west),
                };
                let tile = match edge {
                    Edge::South => g.tile_width,
                    _ => g.tile_height,
                };
                on_edge.then_some((
                    unit + offset as f32 * tile * 2.,
                    unit + (offset + 1) as f32 * tile * 2.,
                ))
            })
            .collect::<Vec<_>>();
        if matches!(edge, Edge::East) {
            gaps.extend(native_radio_studio::perimeter_glass(
                &frame.map_id,
                cells,
                g,
                placements,
                &mesh.authored_cells,
                frame.grid_origin.to_array(),
                width,
            ));
        }
        gaps.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut start = from;
        while start < to - 0.001 {
            let end = (start + step).min(to);
            let mut runs = vec![(start, end)];
            for &(a, b) in &gaps {
                runs = runs
                    .into_iter()
                    .flat_map(|(s, e)| {
                        if b <= s || a >= e {
                            vec![(s, e)]
                        } else {
                            [(s, a.min(e)), (b.max(s), e)]
                                .into_iter()
                                .filter(|(l, r)| r - l > 0.001)
                                .collect()
                        }
                    })
                    .collect();
            }
            for (s, e) in runs {
                if e < grid_from || s > grid_to {
                    continue;
                }
                let first_solid = mesh.solid.positions.len();
                let first_textured = mesh.textured.positions.len();
                copy_section(&mut mesh.solid, &solid, b, edge, at, s, e);
                copy_section(&mut mesh.textured, &textured, b, edge, at, s, e);
                let plane = match edge {
                    Edge::West => [1., 0., at],
                    Edge::East => [-1., 0., -at],
                    Edge::South => [0., -1., -at],
                };
                if first_solid < mesh.solid.positions.len() {
                    mesh.camera_wall_solid_ranges
                        .push((first_solid..mesh.solid.positions.len(), plane));
                }
                if first_textured < mesh.textured.positions.len() {
                    mesh.camera_wall_textured_ranges
                        .push((first_textured..mesh.textured.positions.len(), plane));
                }
            }
            start = end;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn external_room_wall_donors() {
        let path = std::env::var_os("CRYSTAL_RENDER_TEST_PACK")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../content-packs/text-tui.crystalpack")
            });
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let pack = loaded.pack();
        let mut total = 0;
        let mut missing = std::collections::BTreeMap::<String, Vec<String>>::new();
        for (map, data) in &pack.data().maps {
            let tileset = pack.data().map_tileset_name(map).unwrap();
            if !crate::interior::has_room_perimeter(map, &tileset) {
                continue;
            }
            if matches!(
                map.as_str(),
                "GoldenrodDeptStoreRoof" | "CeladonMansionRoof" | "TinTowerRoof"
            ) {
                continue;
            }
            total += 1;
            let art = &pack.runtime_files()[&format!("data/tilesets/{tileset}_metatiles.bin")];
            let (w, h) = (
                data.attributes.width as usize * 4,
                data.attributes.height as usize * 4,
            );
            let g = GridGeometry {
                width: w,
                height: h,
                tile_width: 8.,
                tile_height: 8.,
                origin_x: 0.,
                origin_z: 0.,
            };
            let tiles = (0..w * h)
                .map(|i| {
                    let (x, y) = (i % w, i / w);
                    let b = data.blocks[y / 4 * (w / 4) + x / 4];
                    VisualTile {
                        column: x as u32,
                        row: y as u32,
                        texture: Default::default(),
                        priority: false,
                        animation_frames: None,
                        source: VisualTileSource {
                            tileset_id: tileset.into(),
                            metatile_id: b,
                            subtile_column: (x % 4) as u8,
                            subtile_row: (y % 4) as u8,
                            tile_index: art[b as usize * 16 + y % 4 * 4 + x % 4] as u16,
                        },
                    }
                })
                .collect::<Vec<_>>();
            let refs = tiles.iter().collect::<Vec<_>>();
            let p = resolve(map, &refs, &g, None);
            if map == "KurtsHouse" {
                let mut timber = SurfaceMeshData::default();
                assert!(
                    traditional_wood_wall::perimeter_template(&mut timber, map, &refs, &g, &p)
                        .is_some()
                );
                let swatches = timber
                    .uvs
                    .iter()
                    .map(|uv| uv.map(f32::to_bits))
                    .collect::<std::collections::BTreeSet<_>>();
                assert!(
                    swatches.len() >= 3,
                    "timber returns keep their native grain and joints"
                );
            }
            if map == "VioletPokecenter1F" {
                let donor = p
                    .iter()
                    .filter(|p| is_wall(p.kind) && p.base_pixels == 0. && p.depth_pixels <= 16.)
                    .min_by_key(|p| {
                        (
                            p.kind == ModelKind::WindowWall,
                            p.row,
                            !plain_panel(p, &refs, &g),
                            p.column,
                        )
                    })
                    .unwrap();
                assert!(
                    plain_panel(donor, &refs, &g),
                    "Center returns must not repeat wall-mounted displays"
                );
                assert_eq!(refs[donor.row * w + donor.column].source.tile_index, 2);
            }
            let dungeon = super::super::super::modeled_dungeons::resolve(map, &refs, &g, None);
            let mut frame = super::super::super::tests::frame(
                w as u32,
                h as u32,
                tiles.iter().map(|t| t.source.clone()).collect(),
            );
            frame.map_id = map.as_str().into();
            frame.source_map_size_core_tiles = Some(UVec2::new(w as u32 / 2, h as u32 / 2));
            frame.boundary_warps = data
                .events
                .warps
                .iter()
                .map(|warp| IVec2::new(i32::from(warp.x), i32::from(warp.y)))
                .collect();
            let mut mesh = TerrainMeshData {
                footing_heights: vec![2.75; w * h],
                authored_cells: vec![None; w * h],
                ..Default::default()
            };
            append(&mut mesh, &frame, &refs, &g, &p, &dungeon, None);
            if mesh.solid.indices.is_empty() && mesh.textured.indices.is_empty() {
                missing
                    .entry(tileset.to_string())
                    .or_default()
                    .push(map.clone());
            } else {
                let inset = tin_room_bounds::insets(&frame, &refs, &g, w as i32, h as i32);
                if super::super::super::modeled_dungeons::tin_tower_guardian_map(map) {
                    assert_eq!(inset, [4;3], "all nine tower floors use the inner padding edge");
                    let mut altered = tiles.clone();
                    altered[w * 4].source.tile_index = 2;
                    let altered_refs = altered.iter().collect::<Vec<_>>();
                    assert_eq!(tin_room_bounds::insets(&frame, &altered_refs, &g, w as i32, h as i32),
                        [0,4,4], "an edited boundary is never inferred as blank padding");
                    let mut other = frame.clone();
                    other.map_id = "SproutTower1F".into();
                    assert_eq!(tin_room_bounds::insets(&other, &refs, &g, w as i32, h as i32),
                        [0;3], "other tower architecture retains its original boundaries");
                    for (surface, ranges) in [(&mesh.solid, &mesh.camera_wall_solid_ranges),
                        (&mesh.textured, &mesh.camera_wall_textured_ranges)] {
                        for (range, plane) in ranges {
                            if plane[0] > 0. {
                                assert_eq!(plane[2], 32., "Tin west wall meets source column four");
                                assert!(surface.positions[range.clone()].iter().any(|p| (p[0]-32.).abs()<0.001));
                            }
                        }
                    }
                }
                let points = mesh
                    .solid
                    .positions
                    .iter()
                    .chain(&mesh.textured.positions)
                    .collect::<Vec<_>>();
                assert!(
                    points.iter().any(|p| p[0] <= inset[0] as f32 * 8. && p[1] > 8.),
                    "{map}: west wall"
                );
                assert!(
                    points.iter().any(|p| p[0] >= (w as i32 - inset[1]) as f32 * 8. && p[1] > 8.),
                    "{map}: east wall"
                );
                assert!(
                    points.iter().any(|p| p[2] >= (h as i32 - inset[2]) as f32 * 8. && p[1] > 8.),
                    "{map}: front wall"
                );
                assert_eq!(
                    mesh.footing_heights,
                    vec![2.75; w * h],
                    "{map}: floors remain authoritative"
                );
                assert_eq!(
                    mesh.authored_cells,
                    vec![None; w * h],
                    "{map}: no source cells replaced"
                );
                for surface in [&mesh.solid, &mesh.textured] {
                    assert_eq!(surface.positions.len(), surface.normals.len());
                    assert_eq!(surface.positions.len(), surface.uvs.len());
                    assert!(
                        surface
                            .normals
                            .iter()
                            .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 0.001)
                    );
                    assert_eq!(
                        surface
                            .cutaway_ranges
                            .iter()
                            .map(|r| r.len())
                            .sum::<usize>(),
                        surface.positions.len()
                    );
                }
                for (surface, sections) in [
                    (&mesh.solid, &mesh.camera_wall_solid_ranges),
                    (&mesh.textured, &mesh.camera_wall_textured_ranges),
                ] {
                    for (range, plane) in sections {
                        let points = &surface.positions[range.clone()];
                        assert!(
                            points
                                .iter()
                                .all(|p| plane[0] * p[0] + plane[1] * p[2] <= plane[2] + 0.001),
                            "{map}: walls cannot consume playable floor"
                        );
                        for warp in frame.boundary_warps.iter() {
                            let horizontal = plane[1] < 0.;
                            let exit = if horizontal {
                                warp.y >= (h as i32 - inset[2]) / 2 - 2
                            } else if plane[0] > 0. {
                                warp.x == inset[0] / 2
                            } else {
                                warp.x == (w as i32 - inset[1]) / 2 - 1
                            };
                            if !exit {
                                continue;
                            }
                            let axis = if horizontal { 0 } else { 2 };
                            let coordinate = if horizontal { warp.x } else { warp.y };
                            let start = coordinate as f32 * 16.;
                            let end = start + 16.;
                            let min = points.iter().map(|p| p[axis]).fold(f32::INFINITY, f32::min);
                            let max = points
                                .iter()
                                .map(|p| p[axis])
                                .fold(f32::NEG_INFINITY, f32::max);
                            assert!(
                                max <= start + 0.001 || min >= end - 0.001,
                                "{map}: boundary warp {warp:?} closed"
                            );
                        }
                    }
                }
            }
        }
        eprintln!("rooms={total} missing={missing:?}");
        assert!(total > 0);
        assert!(
            missing.is_empty(),
            "every enclosed map needs side and front walls: {missing:?}"
        );
    }
}
