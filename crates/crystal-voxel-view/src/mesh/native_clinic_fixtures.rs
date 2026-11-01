//! Complete Center counter terminal, stair flights, and rear equipment backing.
use super::*;

fn lobby(map: &str) -> bool {
    kanto_native_floor_map(map) && map.ends_with("Pokecenter1F")
}

fn station_cell(s: &VisualTileSource) -> Option<(u8, u8)> {
    if s.tileset_id.as_ref() != "pokecenter" || !(2..4).contains(&s.subtile_column) {
        return None;
    }
    let y = match s.metatile_id {
        3 if s.subtile_row < 4 => s.subtile_row,
        7 if s.subtile_row < 2 => s.subtile_row + 4,
        _ => return None,
    };
    let x = s.subtile_column - 2;
    let drawing = [[2, 2], [2, 15], [1, 15], [3, 37], [19, 53], [70, 71]];
    (s.tile_index == drawing[y as usize][x as usize]).then_some((x, y))
}

pub(super) fn station(map: &str, p: &Placement, cells: &[&VisualTile], g: &GridGeometry) -> bool {
    lobby(map)
        && p.kind == ModelKind::Television
        && p.width == 2
        && p.height == 6
        && p.indices(g.width)
            .enumerate()
            .all(|(j, i)| station_cell(&cells[i].source) == Some(((j % 2) as u8, (j / 2) as u8)))
}

fn stair_cell(map: &str, s: &VisualTileSource) -> Option<(u8, u8, bool)> {
    let down = if lobby(map) {
        false
    } else if map == "Pokecenter2F"
        || kanto_native_floor_map(map) && map.ends_with("Pokecenter2FBeta")
    {
        true
    } else {
        return None;
    };
    if s.tileset_id.as_ref() != "pokecenter"
        || s.metatile_id != if down { 0x10 } else { 0x12 }
        || s.subtile_column >= 2
        || !(2..4).contains(&s.subtile_row)
    {
        return None;
    }
    let x = s.subtile_column;
    let y = s.subtile_row - 2;
    let drawing = if down {
        [[0x42, 0x43], [0x52, 0x53]]
    } else {
        [[0x44, 0x45], [0x54, 0x55]]
    };
    (s.tile_index == drawing[y as usize][x as usize]).then_some((x, y, down))
}

pub(super) fn stair(
    map: &str,
    p: &Placement,
    cells: &[&VisualTile],
    g: &GridGeometry,
) -> Option<bool> {
    if p.kind != ModelKind::StairFlight || p.width != 2 || p.height != 2 {
        return None;
    }
    let (_, _, down) = stair_cell(map, &cells[p.row * g.width + p.column].source)?;
    p.indices(g.width)
        .enumerate()
        .all(|(j, i)| {
            stair_cell(map, &cells[i].source) == Some(((j % 2) as u8, (j / 2) as u8, down))
        })
        .then_some(down)
}

pub(super) fn candidates(
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
    claimed: &mut [bool],
    out: &mut Vec<Placement>,
) {
    let Some(ground) = ground_sample(cells, map, "pokecenter", 0x11) else {
        return;
    };
    let mut add = |groups: Vec<TreePlacement>, kind, height, base| {
        for group in groups {
            let p = Placement {
                kind,
                column: group.column,
                row: group.row,
                width: group.width,
                height: group.height,
                ground,
                depth_pixels: 16.,
                height_pixels: height,
                base_pixels: base,
                footing_pixels: None,
            };
            if p.indices(g.width).any(|i| claimed[i]) {
                continue;
            }
            for i in p.indices(g.width) {
                claimed[i] = true;
            }
            out.push(p);
        }
    };
    if lobby(map) {
        add(
            grouped_flat_card_placements(cells, g, 0x11, false, |s| {
                station_cell(s).map(|(x, y)| (x, y, 2, 6))
            }),
            ModelKind::Television,
            12.,
            8.,
        );
    }
    for down in [false, true] {
        add(
            grouped_flat_card_placements(cells, g, 0x11, false, |s| {
                stair_cell(map, s)
                    .filter(|&(_, _, d)| d == down)
                    .map(|(x, y, _)| (x, y, 2, 2))
            }),
            ModelKind::StairFlight,
            12.,
            if down { -16. * 12. / 22.4 } else { 0. },
        );
    }
}

/// Repeat only a verified plain wall tile, with the same 16px courses as the
/// neighboring clinical panels. Equipment must not punch holes through it.
fn wall(
    out: &mut SurfaceMeshData,
    g: &GridGeometry,
    donor: [usize; 2],
    bounds: [f32; 4],
    columns: usize,
    height: f32,
) {
    let [w, e, n, s] = bounds;
    let first = out.positions.len();
    let indices = out.indices.len();
    let uv = g.uv(donor[0], donor[1]);
    let paint = [
        uv.0 + (uv.1 - uv.0) * 4.5 / 8.,
        uv.2 + (uv.3 - uv.2) * 1.5 / 8.,
    ];
    architecture_box(out, [w, e, 0., height, n, s], [1.; 3]);
    out.uvs[first..].fill(paint);
    out.indices.drain(indices + 6..indices + 12);
    let rows = (height / (16. * g.tile_height / 8.)).round() as usize;
    for y in 0..rows {
        for x in 0..columns {
            let left = w + (e - w) * x as f32 / columns as f32;
            let right = w + (e - w) * (x + 1) as f32 / columns as f32;
            let low = height * y as f32 / rows as f32;
            let high = height * (y + 1) as f32 / rows as f32;
            append_quad(
                out,
                [
                    [right, low, s],
                    [right, high, s],
                    [left, high, s],
                    [left, low, s],
                ],
                [0., 0., 1.],
                [[uv.1, uv.3], [uv.1, uv.2], [uv.0, uv.2], [uv.0, uv.3]],
                TEXTURED_SHADE,
            );
        }
    }
}

pub(super) fn append_backing(
    out: &mut SurfaceMeshData,
    map: &str,
    p: &Placement,
    cells: &[&VisualTile],
    g: &GridGeometry,
    placements: &[Placement],
) {
    let pc = native_clinic::pc_source(map, p, cells, g);
    let stock = native_clinic::stock_source(map, p, cells, g);
    if !pc
        && !stock
        && !native_clinic::healing_source(map, p, cells, g)
        && !native_clinic::cabinet_source(map, p, cells, g)
    {
        return;
    }
    let Some(donor) = placements.iter().find(|d| {
        d.height == 2
            && (!stock || d.row == p.row)
            && native_clinic::wall_source(map, d, cells, g)
            && d.indices(g.width).all(|i| cells[i].source.tile_index == 2)
    }) else {
        return;
    };
    let scale = g.tile_height / 8.;
    let w = g.origin_x + p.column as f32 * g.tile_width;
    let e = w + p.width as f32 * g.tile_width;
    let s = g.origin_z + (donor.row + 2) as f32 * g.tile_height;
    let header = pc
        && placements.iter().any(|d| {
            d.column == p.column
                && d.row + 1 == p.row
                && d.height == 1
                && native_clinic::wall_source(map, d, cells, g)
        });
    wall(
        out,
        g,
        [donor.column, donor.row],
        [w, e, s - 3. * scale, s],
        p.width,
        if header { 16. } else { 32. } * scale,
    );
}

pub(super) fn append_station(
    out: &mut SurfaceMeshData,
    g: &GridGeometry,
    p: &Placement,
    selected: &crate::interior_models::Model,
) {
    let sx = g.tile_width / 8.;
    let sy = g.tile_height / 8.;
    let w = g.origin_x + p.column as f32 * g.tile_width;
    let e = w + 2. * g.tile_width;
    let rear = g.origin_z + (p.row + 2) as f32 * g.tile_height;
    let s = g.origin_z + (p.row + 6) as f32 * g.tile_height;
    wall(
        out,
        g,
        [p.column, p.row],
        [w, e, rear - 3. * sy, rear],
        2,
        32. * sy,
    );
    let [white, mid, blue, ink] =
        native_source_swatches(p, g, [[2, 32], [1, 32], [4, 32], [0, 32]]);
    let mut painted_box = |b, paint| {
        let first = out.positions.len();
        architecture_box(out, b, [1.; 3]);
        out.uvs[first..].fill(paint);
    };
    // One low divider joins the rear wall to the counter. The source's vertical
    // perspective strip is not a second wall-height monitor.
    painted_box([e - 5. * sx, e, 0., 8. * sy, rear, s - 8. * sy], blue);
    painted_box(
        [e - 5. * sx, e, 8. * sy, 8.5 * sy, rear, s - 8. * sy],
        white,
    );
    painted_box([w, e, 0., 7. * sy, s - 8. * sy, s], blue);
    painted_box([w, e, 7. * sy, 8. * sy, s - 8. * sy, s], white);
    let first = out.positions.len();
    assert!(selected.append_native_television_screen(
        out,
        [w + 2. * sx, e - 2. * sx, s - 8. * sy, s - 0.5 * sy],
        12. * sy,
        [white, white, mid, ink, blue]
    ));
    for v in &mut out.positions[first..] {
        v[1] += 8. * sy;
    }
}

pub(super) fn append_stair(
    out: &mut SurfaceMeshData,
    g: &GridGeometry,
    p: &Placement,
    bounds: [f32; 4],
    down: bool,
    selected: &crate::interior_models::Model,
) {
    let pixels = if down {
        [[0, 0], [0, 0], [1, 0], [0, 1], [0, 1]]
    } else {
        [[1, 1], [1, 1], [7, 0], [3, 0], [3, 0]]
    };
    let first = out.positions.len();
    assert!(selected.append_native_stair(
        out,
        bounds,
        p.base_pixels * g.tile_height / 8.,
        p.height_pixels * g.tile_height / 8.,
        native_source_swatches(p, g, pixels)
    ));
    if !down {
        // The lobby entrance is east of the warp: the flight rises west.
        let [w, e, n, s] = bounds;
        for v in &mut out.positions[first..] {
            v[0] = w + e - v[0];
            v[2] = n + s - v[2];
        }
        for (normal, color) in out.normals[first..]
            .iter_mut()
            .zip(&mut out.colors[first..])
        {
            normal[0] = -normal[0];
            normal[2] = -normal[2];
            *color =
                crate::interior_models::authored_face_color([1.; 4], Vec3::from_array(*normal));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pack() -> Option<crystal_assets::LoadedCompiledGamePack> {
        let explicit = std::env::var_os("CRYSTAL_RENDER_TEST_PACK");
        let path = explicit
            .as_ref()
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../content-packs/text-tui.crystalpack")
            });
        assert!(explicit.is_none() || path.is_file());
        path.is_file()
            .then(|| crystal_assets::read_loaded_verified_compiled_game_pack(path).unwrap())
    }

    fn room(
        pack: &crystal_assets::LoadedCompiledGamePack,
        map: &str,
    ) -> (Vec<VisualTile>, GridGeometry) {
        let data = &pack.pack().data().maps[map];
        let layout = &pack.pack().runtime_files()["data/tilesets/pokecenter_metatiles.bin"];
        let (w, h) = (
            data.attributes.width as usize * 4,
            data.attributes.height as usize * 4,
        );
        let tiles = (0..w * h)
            .map(|i| {
                let (x, y) = (i % w, i / w);
                let block = data.blocks[y / 4 * (w / 4) + x / 4];
                VisualTile {
                    column: x as u32,
                    row: y as u32,
                    texture: Default::default(),
                    priority: false,
                    animation_frames: None,
                    source: VisualTileSource {
                        tileset_id: "pokecenter".into(),
                        metatile_id: block as u16,
                        subtile_column: (x % 4) as u8,
                        subtile_row: (y % 4) as u8,
                        tile_index: layout[block as usize * 16 + y % 4 * 4 + x % 4] as u16,
                    },
                }
            })
            .collect();
        (
            tiles,
            GridGeometry {
                width: w,
                height: h,
                tile_width: 8.,
                tile_height: 8.,
                origin_x: 0.,
                origin_z: 0.,
            },
        )
    }

    #[test]
    fn external_indigo_stock_keeps_two_cabinets_and_closes_the_same_rear_wall_course() {
        let Some(pack)=pack() else{return;};
        let map="IndigoPlateauPokecenter1F";
        let (tiles,g)=room(&pack,map);let refs=tiles.iter().collect::<Vec<_>>();
        let placements=resolve(map,&refs,&g,Some(built_in_profiles()));
        let stock=placements.iter().filter(|p|native_clinic::stock_source(map,p,&refs,&g)).collect::<Vec<_>>();
        assert_eq!(stock.len(),2);
        assert_eq!(stock.iter().map(|p|(p.column,p.row)).collect::<Vec<_>>(),vec![(28,12),(30,12)]);
        for p in stock {
            let mut out=SurfaceMeshData::default();
            append_backing(&mut out,map,p,&refs,&g,&placements);
            assert!(!out.indices.is_empty());
            assert_eq!(out.positions.iter().map(|v|v[1]).fold(0.,f32::max),32.);
            assert_eq!(out.positions.iter().map(|v|v[2]).fold(0.,f32::max),112.,
                "continue the wall at source row14, not the stair alcove wall at row2");
            assert_eq!(out.positions.iter().map(|v|v[0]).fold(f32::INFINITY,f32::min),p.column as f32*8.);
            assert_eq!(out.positions.iter().map(|v|v[0]).fold(0.,f32::max),(p.column+2)as f32*8.);
            assert!(placements.iter().filter(|q|!std::ptr::eq(*q,p))
                .all(|q|!q.indices(g.width).any(|i|p.indices(g.width).any(|j|i==j))));
        }
    }
    #[test]
    fn external_clinic_terminal_is_one_complete_assembly_without_a_second_screen_surface() {
        let Some(pack) = pack() else {
            return;
        };
        for map in [
            "CherrygrovePokecenter1F",
            "GoldenrodPokecenter1F",
            "PewterPokecenter1F",
        ] {
            let (tiles, g) = room(&pack, map);
            let refs = tiles.iter().collect::<Vec<_>>();
            let placements = resolve(map, &refs, &g, Some(built_in_profiles()));
            let terminals = placements
                .iter()
                .filter(|p| station(map, p, &refs, &g))
                .collect::<Vec<_>>();
            assert_eq!(terminals.len(), 1);
            let p = terminals[0];
            assert!(
                !placements
                    .iter()
                    .filter(|other| !std::ptr::eq(*other, p))
                    .any(|other| other
                        .indices(g.width)
                        .any(|i| p.indices(g.width).any(|j| i == j)))
            );
            let mut mesh = TerrainMeshData {
                footing_heights: vec![1.25; tiles.len()],
                ..Default::default()
            };
            let mut claims = vec![false; tiles.len()];
            assert!(append_with_tables(
                &mut mesh,
                map,
                &refs,
                &g,
                p,
                &mut claims,
                [0, 0],
                &placements
            ));
            assert_eq!(claims.iter().filter(|&&v| v).count(), 12);
            assert!(mesh.solid.indices.is_empty());
            assert_eq!(mesh.footing_heights, vec![1.25; tiles.len()]);
            // Every furnishing face uses a material sample. A whole source
            // screen mapped onto either the desktop or counter front fails.
            let rear = (p.row + 2) as f32 * 8.;
            for tri in mesh.textured.indices[12 * 6..].chunks_exact(3) {
                let uv = [tri[0], tri[1], tri[2]].map(|i| mesh.textured.uvs[i as usize]);
                if uv[0] != uv[1] || uv[1] != uv[2] {
                    assert!(
                        tri.iter()
                            .all(|&i| mesh.textured.positions[i as usize][2] == rear)
                    );
                }
            }
            for i in p.indices(g.width) {
                for field in 0..4 {
                    let mut changed = tiles.clone();
                    let s = &mut changed[i].source;
                    match field {
                        0 => s.tile_index ^= 1,
                        1 => s.metatile_id ^= 1,
                        2 => s.subtile_column ^= 1,
                        _ => s.subtile_row ^= 1,
                    }
                    assert!(
                        !resolve(map, &changed.iter().collect::<Vec<_>>(), &g, None)
                            .iter()
                            .any(|q| station(map, q, &changed.iter().collect::<Vec<_>>(), &g))
                    );
                }
            }
            let custom = Document {
                objects: vec![Object {
                    name: "Custom Center terminal".into(),
                    ..built_in_profiles()
                        .objects
                        .iter()
                        .find(|o| o.name == "Shared Pokecenter complete PC")
                        .unwrap()
                        .clone()
                }],
                atmosphere: None,
            };
            // An explicit custom owner over the monitor's lower source cells
            // prevents the compiled whole assembly from taking those cells.
            let mut custom = custom;
            custom.objects[0].metatile = 7;
            custom.objects[0].origin = [2, 0];
            custom.objects[0].tiles = vec![vec![19, 53], vec![70, 71]];
            assert!(
                !resolve(map, &refs, &g, Some(&custom))
                    .iter()
                    .any(|q| station(map, q, &refs, &g))
            );
        }
    }

    #[test]
    fn external_clinic_stairs_have_treads_and_closed_backing_without_changing_support() {
        let Some(pack) = pack() else {
            return;
        };
        for (map, down) in [
            ("CherrygrovePokecenter1F", false),
            ("PewterPokecenter1F", false),
            ("Pokecenter2F", true),
            ("PewterPokecenter2FBeta", true),
        ] {
            let (tiles, g) = room(&pack, map);
            let refs = tiles.iter().collect::<Vec<_>>();
            let placements = resolve(map, &refs, &g, Some(built_in_profiles()));
            let stairs = placements
                .iter()
                .filter(|p| stair(map, p, &refs, &g) == Some(down))
                .collect::<Vec<_>>();
            assert_eq!(stairs.len(), 1, "{map}");
            let p = stairs[0];
            let mut mesh = TerrainMeshData {
                footing_heights: vec![0.; tiles.len()],
                ..Default::default()
            };
            let mut claims = vec![false; tiles.len()];
            assert!(append_with_tables(
                &mut mesh,
                map,
                &refs,
                &g,
                p,
                &mut claims,
                [0, 0],
                &placements
            ));
            assert_eq!(claims.iter().filter(|&&v| v).count(), 4);
            assert_eq!(mesh.footing_heights, vec![0.; tiles.len()]);
            assert!(mesh.solid.indices.is_empty());
            let mut heights = mesh.textured.positions[16..]
                .iter()
                .zip(&mesh.textured.normals[16..])
                .filter(|(_, n)| n[1] > 0.99)
                .map(|(v, _)| v[1])
                .collect::<Vec<_>>();
            heights.sort_by(f32::total_cmp);
            heights.dedup_by(|a, b| (*a - *b).abs() < 0.001);
            assert!(heights.len() >= 6, "{map}: separate raised treads");
            assert!(if down {
                heights[0] < -1.
            } else {
                *heights.last().unwrap() > 6.
            });
            for tri in mesh.textured.indices[4 * 6..].chunks_exact(3) {
                let [a, b, c] = [tri[0], tri[1], tri[2]]
                    .map(|i| Vec3::from_array(mesh.textured.positions[i as usize]));
                let n = Vec3::from_array(mesh.textured.normals[tri[0] as usize]);
                assert!((b - a).cross(c - a).dot(n) > 0.);
            }
            for i in p.indices(g.width) {
                let mut changed = tiles.clone();
                changed[i].source.tile_index ^= 1;
                let refs = changed.iter().collect::<Vec<_>>();
                assert!(
                    !resolve(map, &refs, &g, None)
                        .iter()
                        .any(|q| stair(map, q, &refs, &g).is_some())
                );
            }
        }
    }

    #[test]
    fn external_clinic_walls_close_the_healing_machine_and_pc_gaps_at_the_rear_seam() {
        let Some(pack) = pack() else {
            return;
        };
        for map in ["CherrygrovePokecenter1F", "PewterPokecenter1F"] {
            let (tiles, g) = room(&pack, map);
            let refs = tiles.iter().collect::<Vec<_>>();
            let placements = resolve(map, &refs, &g, None);
            for kind in [ModelKind::Computer, ModelKind::HealingMachine] {
                let p = placements.iter().find(|p| p.kind == kind).unwrap();
                let mut backing = SurfaceMeshData::default();
                append_backing(&mut backing, map, p, &refs, &g, &placements);
                let faces = backing
                    .indices
                    .chunks_exact(3)
                    .filter(|t| backing.normals[t[0] as usize] == [0., 0., 1.])
                    .map(|t| {
                        [t[0], t[1], t[2]].map(|i| Vec3::from_array(backing.positions[i as usize]))
                    })
                    .collect::<Vec<_>>();
                assert!(!faces.is_empty());
                let area: f32 = faces
                    .iter()
                    .map(|[a, b, c]| (*b - *a).cross(*c - *a).length() * 0.5)
                    .sum();
                let height = if kind == ModelKind::Computer {
                    16.
                } else {
                    32.
                };
                assert!((area - p.width as f32 * 8. * height).abs() < 0.001);
                assert!(
                    faces
                        .iter()
                        .flatten()
                        .all(|v| v.z == 16. && v.y >= 0. && v.y <= height)
                );
            }
        }
    }
}
