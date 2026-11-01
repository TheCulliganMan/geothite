//! A domestic flight sits in a recessed rear wall, with its south approach open.
use super::*;

fn scope(map: &str) -> bool {
    matches!(
        map,
        "PlayersHouse1F"
            | "PlayersHouse2F"
            | "RedsHouse1F"
            | "RedsHouse2F"
            | "CopycatsHouse1F"
            | "CopycatsHouse2F"
    )
}

pub(in crate::mesh::modeled_interiors) fn append(
    mesh: &mut TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    origin: [i32; 2],
    excluded: &[bool],
) {
    if !scope(map) {
        return;
    }
    if map.starts_with("PlayersHouse") {
        let (Ok(column), Ok(row)) = (usize::try_from(-origin[0]), usize::try_from(-origin[1]))
        else {
            return;
        };
        let width = if map == "PlayersHouse2F" { 16 } else { 20 };
        let complete = if map == "PlayersHouse2F" {
            native_bedroom_backing_source(map, cells, g, column, row)
        } else {
            native_living_room_backing_source(map, cells, g, column, row)
        };
        if !complete
            || (0..4).any(|y| {
                (0..width).any(|x| excluded.get((row + y) * g.width + column + x) == Some(&true))
            })
        {
            return;
        }
    }
    let placements = resolve(map, cells, g, None);
    let protected = |p: &Placement| p.indices(g.width).any(|i| excluded.get(i) == Some(&true));
    for stair in placements
        .iter()
        .filter(|p| p.kind == ModelKind::StairFlight)
    {
        if origin[1] + stair.row as i32 != 0
            || protected(stair)
            || super::super::native_home_stair::source(map, stair, cells, g).is_none()
        {
            continue;
        }
        let atlas = &cells[stair.row * g.width + stair.column].source.tileset_id;
        let plank = if atlas.as_ref() == "players_room" {
            2
        } else {
            0x11
        };
        let donor = placements
            .iter()
            .filter(|p| {
                p.kind == ModelKind::WallDomestic
                    && p.height == 2
                    && p.width >= 2
                    && p.base_pixels == 0.
                    && origin[1] + p.row as i32 == 0
                    && !protected(p)
                    && p.indices(g.width).all(|i| {
                        cells[i].source.tileset_id == *atlas && cells[i].source.tile_index == plank
                    })
            })
            .min_by_key(|p| p.column.abs_diff(stair.column))
            .cloned()
            .or_else(|| {
                // The living room's plain block05 is not a separate prop.
                // Authenticate its complete rear drawing, including profile ownership.
                if map != "PlayersHouse1F" || g.width < 4 || g.height < 2 {
                    return None;
                }
                for row in 0..=g.height - 2 {
                    for column in 0..=g.width - 4 {
                        if origin[1] + row as i32 != 0 {
                            continue;
                        }
                        if (0..2).all(|y| {
                            (0..4).all(|x| {
                                let i = (row + y) * g.width + column + x;
                                let s = &cells[i].source;
                                excluded.get(i) != Some(&true)
                                    && s.tileset_id.as_ref() == "players_house"
                                    && s.metatile_id == 5
                                    && s.tile_index == 0x11
                                    && s.subtile_column == x as u8
                                    && s.subtile_row == y as u8
                            })
                        }) {
                            return Some(Placement {
                                kind: ModelKind::WallDomestic,
                                column,
                                row,
                                width: 4,
                                height: 2,
                                ground: row * g.width + column,
                                depth_pixels: 3.,
                                height_pixels: 32.,
                                base_pixels: 0.,
                                footing_pixels: None,
                            });
                        }
                    }
                }
                None
            });
        let Some(mut donor) = donor else {
            continue;
        };
        donor.width = 2;
        donor.base_pixels = stair.base_pixels.min(0.);
        donor.height_pixels = 32. - donor.base_pixels;
        let w = g.origin_x + stair.column as f32 * g.tile_width;
        let e = w + 2. * g.tile_width;
        let back = g.origin_z + stair.row as f32 * g.tile_height;
        // The boundary warp leaves a full two-cell interval in the side shell.
        // Reach its end, overlapping the slightly recessed rear wall course.
        let front = back + 2. * g.tile_height;
        let thickness = 3. * g.tile_height / 8.;
        let bounds = [0., 2. * g.tile_width, -thickness, 0.];
        let mut template = SurfaceMeshData::default();
        let append_wall=|out:&mut SurfaceMeshData,bounds| {
            if map.starts_with("PlayersHouse") {
                native_home_timber::append(out,g,bounds,donor.base_pixels*g.tile_height/8.,32.*g.tile_height/8.,[donor.column,donor.row]);
            } else {append_kanto_house_wall(out,g,&donor,bounds);}
        };
        append_wall(&mut template,bounds);
        let first = mesh.textured.positions.len();
        append_wall(&mut mesh.textured,[w,e,back-thickness,back]);
        mesh.textured
            .cutaway_ranges
            .push(first..mesh.textured.positions.len());
        // These short returns are interior partitions, not the room envelope.
        // Ordinary ray occlusion hides them only when they cover the player;
        // an outside-half-plane rule would open a false gap beside the flight.
        for (edge, at) in [(Edge::West, w), (Edge::East, e)] {
            copy_section(
                &mut mesh.textured,
                &template,
                bounds,
                edge,
                at,
                back - thickness,
                front,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn external_home_stair_recess_closes_three_sides_and_keeps_the_approach_open() {
        let path = std::env::var_os("CRYSTAL_RENDER_TEST_PACK").expect("external test pack");
        let loaded =
            crystal_assets::read_loaded_verified_compiled_game_pack(std::path::PathBuf::from(path))
                .unwrap();
        let pack = loaded.pack();
        for map in [
            "RedsHouse1F",
            "RedsHouse2F",
            "CopycatsHouse1F",
            "CopycatsHouse2F",
            "PlayersHouse1F",
            "PlayersHouse2F",
        ] {
            let data = &pack.data().maps[map];
            let ts = data.attributes.tileset_name.as_str();
            let art = &pack.runtime_files()[&format!("data/tilesets/{ts}_metatiles.bin")];
            let w = data.attributes.width as usize * 4;
            let h = data.attributes.height as usize * 4;
            let tiles = (0..w * h)
                .map(|i| {
                    let x = i % w;
                    let y = i / w;
                    let b = data.blocks[y / 4 * (w / 4) + x / 4];
                    VisualTile {
                        column: x as u32,
                        row: y as u32,
                        texture: Default::default(),
                        priority: false,
                        animation_frames: None,
                        source: VisualTileSource {
                            tileset_id: ts.into(),
                            metatile_id: b,
                            subtile_column: (x % 4) as u8,
                            subtile_row: (y % 4) as u8,
                            tile_index: art[b as usize * 16 + y % 4 * 4 + x % 4] as u16,
                        },
                    }
                })
                .collect::<Vec<_>>();
            let cells = tiles.iter().collect::<Vec<_>>();
            let g = GridGeometry {
                width: w,
                height: h,
                tile_width: 8.,
                tile_height: 8.,
                origin_x: 0.,
                origin_z: 0.,
            };
            let build = |cells: &[&VisualTile], g: &GridGeometry, origin, excluded: &[bool]| {
                let count = g.width * g.height;
                let mut mesh = TerrainMeshData {
                    footing_heights: vec![2.75; count],
                    authored_cells: vec![None; count],
                    ..Default::default()
                };
                append(&mut mesh, map, cells, g, origin, excluded);
                assert_eq!(mesh.footing_heights, vec![2.75; count]);
                assert_eq!(mesh.authored_cells, vec![None; count]);
                mesh
            };
            let placements = resolve(map, &cells, &g, None);
            if let Some((block, phase)) = match map {
                "RedsHouse1F" => Some((0x0a, 0)),
                "CopycatsHouse1F" => Some((0x23, 2)),
                _ => None,
            } {
                let wall = placements
                    .iter()
                    .find(|p| {
                        p.kind == ModelKind::WallDomestic
                            && p.width == 2
                            && p.height == 2
                            && cells[p.row * w + p.column].source.metatile_id == block
                            && cells[p.row * w + p.column].source.subtile_column == phase
                    })
                    .expect("complete wall shoulder beside the flight");
                let mut body = TerrainMeshData {
                    footing_heights: vec![2.75; w * h],
                    authored_cells: vec![None; w * h],
                    ..Default::default()
                };
                let mut claims = vec![false; w * h];
                assert!(crate::mesh::modeled_interiors::append(
                    &mut body,
                    map,
                    &cells,
                    &g,
                    wall,
                    &mut claims,
                    [0, 0]
                ));
                assert_eq!(claims.iter().filter(|&&v| v).count(), 4);
                assert!(
                    body.textured
                        .positions
                        .iter()
                        .any(|p| p[1] == 32. && (p[2] - 16.).abs() < 0.001),
                    "the shoulder meets the neighboring ceiling-height wall course"
                );
                assert_eq!(body.footing_heights, vec![2.75; w * h]);
            }
            let stair = placements
                .iter()
                .find(|p| p.kind == ModelKind::StairFlight)
                .unwrap();
            let mesh = build(&cells, &g, [0, 0], &[]);
            assert!(
                mesh.camera_wall_textured_ranges.is_empty(),
                "{map}: internal recess panels must use player occlusion, not room-edge cutaway"
            );
            assert_eq!(mesh.textured.cutaway_ranges.len(), 3);
            let west = stair.column as f32 * 8.;
            let east = west + 16.;
            let front = 16.;
            let low = stair.base_pixels.min(0.);
            for p in &mesh.textured.positions {
                assert!(
                    p[0] <= west || p[0] >= east || p[2] <= 0.,
                    "{map}: wall cannot cross a tread or its south approach"
                );
                assert!(p[1] >= low && p[1] <= 32. && p[2] <= front + 0.001);
            }
            for range in &mesh.textured.cutaway_ranges {
                let v = &mesh.textured.positions[range.clone()];
                assert_eq!(v.iter().map(|p| p[1]).fold(f32::INFINITY, f32::min), low);
                assert_eq!(
                    v.iter().map(|p| p[1]).fold(f32::NEG_INFINITY, f32::max),
                    32.
                );
            }
            for range in &mesh.textured.cutaway_ranges[1..] {
                let end = mesh.textured.positions[range.clone()]
                    .iter()
                    .map(|p| p[2])
                    .fold(f32::NEG_INFINITY, f32::max);
                assert_eq!(
                    end, 16.,
                    "{map}: return reaches the end of the perimeter warp interval"
                );
            }
            // Each cap and vertical face must retain outward winding after rotation.
            for tri in mesh.textured.indices.chunks_exact(3) {
                let [a, b, c] =
                    [0, 1, 2].map(|j| Vec3::from_array(mesh.textured.positions[tri[j] as usize]));
                assert!(
                    (b - a)
                        .cross(c - a)
                        .dot(Vec3::from_array(mesh.textured.normals[tri[0] as usize]))
                        > 0.
                );
            }
            for i in stair.indices(w) {
                let mut changed = tiles.clone();
                changed[i].source.tile_index = 0xffff;
                assert!(
                    build(&changed.iter().collect::<Vec<_>>(), &g, [0, 0], &[])
                        .textured
                        .indices
                        .is_empty()
                );
                let mut excluded = vec![false; w * h];
                excluded[i] = true;
                assert!(
                    build(&cells, &g, [0, 0], &excluded)
                        .textured
                        .indices
                        .is_empty(),
                    "custom flight retains ownership"
                );
            }
            let mut excluded = vec![true; w * h];
            for i in stair.indices(w) {
                excluded[i] = false;
            }
            assert!(
                build(&cells, &g, [0, 0], &excluded)
                    .textured
                    .indices
                    .is_empty(),
                "custom wall retains ownership"
            );
            assert!(
                build(&cells, &g, [0, 4], &[]).textured.indices.is_empty(),
                "no fake rear boundary on scrolling"
            );
            let mut other = TerrainMeshData::default();
            append(&mut other, "UnreviewedHouse", &cells, &g, [0, 0], &[]);
            assert!(other.textured.indices.is_empty());
            // A padded viewport must produce the exact same physical recess.
            let pw = w + 4;
            let ph = h + 4;
            let mut padding = tiles[0].clone();
            padding.source.metatile_id = u16::MAX;
            padding.source.tile_index = u16::MAX;
            let mut padded = vec![padding; pw * ph];
            for y in 0..h {
                for x in 0..w {
                    let mut t = tiles[y * w + x].clone();
                    t.column += 2;
                    t.row += 2;
                    padded[(y + 2) * pw + x + 2] = t;
                }
            }
            let pg = GridGeometry {
                width: pw,
                height: ph,
                origin_x: -16.,
                origin_z: -16.,
                ..g
            };
            let padded = build(&padded.iter().collect::<Vec<_>>(), &pg, [-2, -2], &[]);
            assert_eq!(
                padded.textured.positions, mesh.textured.positions,
                "{map}: stable wall position on scroll"
            );
        }
    }
}
