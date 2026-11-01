//! Complete native stone flights. Warp plots retain their original support.
use super::*;

fn local(s: &VisualTileSource) -> Option<(u8, u8)> {
    if s.tileset_id.as_ref() != "lighthouse"
        || !(2..4).contains(&s.subtile_column)
        || !(2..4).contains(&s.subtile_row)
    {
        return None;
    }
    let art = match s.metatile_id {
        0x31 => [[0x27, 0x28], [0x37, 0x38]],
        0x3a => [[0x29, 0x2a], [0x39, 0x3a]],
        _ => return None,
    };
    let (x, y) = (s.subtile_column - 2, s.subtile_row - 2);
    (s.tile_index == art[y as usize][x as usize]).then_some((x, y))
}
pub(super) fn resolve_into(r: &mut Resolver<'_>) {
    if !lighthouse_map(r.map) {
        return;
    }
    let Some(ground) = ground(r.map, r.cells, "lighthouse", 0x2e) else {
        return;
    };
    let groups = grouped_flat_card_placements(r.cells, r.g, 0x2e, false, |s| {
        local(s).map(|(x, y)| (x, y, 2, 2))
    });
    for p in groups {
        if !phase_consistent(r.cells, r.g, p) {
            continue;
        }
        r.add(Placement {
            column: p.column,
            row: p.row,
            width: 2,
            height: 2,
            ground,
            form: Form::LighthouseStair {
                down: r.cells[p.row * r.g.width + p.column].source.metatile_id == 0x31,
            },
            rise_pixels: 16.,
            depth_pixels: 16.,
            front_rows: 2.,
        });
    }
}
/// Actual tread contact for complete accepted flights. The stored source
/// support array remains unchanged; this is only the rendered actor's footing.
pub(super) fn support_sample(
    frame: &VisualWorldFrame,
    cell: bevy::prelude::UVec2,
    foot: bevy::prelude::Vec2,
    resolved: &[f32],
) -> Option<f32> {
    if !lighthouse_map(&frame.map_id)
        || !foot.is_finite()
        || !frame.center.is_finite()
        || !frame.tile_size.is_finite()
        || frame.tile_size.min_element() <= 0.
    {
        return None;
    }
    let (width, height) = (frame.grid_size.x as usize, frame.grid_size.y as usize);
    if cell.x as usize >= width || cell.y as usize >= height {
        return None;
    }
    let tile = frame
        .tiles
        .iter()
        .find(|t| t.column == cell.x && t.row == cell.y)?;
    local(&tile.source)?; // Ordinary floor/foreign drawings take the fast path.
    let mut ordered = vec![None; width.checked_mul(height)?];
    for tile in &frame.tiles {
        if tile.column as usize >= width || tile.row as usize >= height {
            return None;
        }
        let index = tile.row as usize * width + tile.column as usize;
        if ordered[index].replace(tile).is_some() {
            return None;
        }
    }
    let cells = ordered.into_iter().collect::<Option<Vec<_>>>()?;
    let g = GridGeometry {
        width,
        height,
        tile_width: frame.tile_size.x,
        tile_height: frame.tile_size.y,
        origin_x: 0.,
        origin_z: 0.,
    };
    // Reuse admission, native floor availability and phase guards from the
    // mesher rather than admitting a lone matching stair subtile.
    let mut resolver = Resolver {
        map: &frame.map_id,
        cells: &cells,
        g: &g,
        claimed: vec![false; cells.len()],
        out: Vec::new(),
    };
    resolve_into(&mut resolver);
    let index = cell.y as usize * width + cell.x as usize;
    let p = resolver
        .out
        .iter()
        .find(|p| p.indices(width).any(|i| i == index))?;
    // A different built support plane must keep its existing presentation.
    if p.indices(width).any(|i| {
        resolved
            .get(i)
            .is_none_or(|h| !h.is_finite() || h.abs() > 0.001)
    }) {
        return None;
    }
    let Form::LighthouseStair { down } = p.form else {
        return None;
    };
    let size = frame.grid_size.as_vec2() * frame.tile_size;
    let x = (foot.x - frame.center.x + size.x * 0.5) / frame.tile_size.x - p.column as f32;
    let z = (frame.center.y + size.y * 0.5 - foot.y) / frame.tile_size.y - p.row as f32;
    if !(0.0..2.0).contains(&x) || !(0.0..2.0).contains(&z) {
        return None;
    }
    let (x, z) = (x * 8., z * 8.);
    let h = if down {
        if z < 1. || z >= 15. || x >= 15. {
            1.
        } else {
            -16. * ((x / 15. * 6.).floor() + 1.).min(6.) / 6.
        }
    } else if z < 1. || z >= 15. {
        16.
    } else if !(1.0..15.0).contains(&x) {
        0.
    } else {
        16. * (((x - 1.) / 14. * 6.).floor() + 1.).min(6.) / 6.
    };
    Some(h * frame.tile_size.y / 8.)
}
pub(super) fn append(mesh: &mut TerrainMeshData, g: &GridGeometry, p: &Placement, down: bool) {
    let (w, _, n, _) = g.bounds(p.column, p.row);
    let k = g.tile_height / 8.;
    let e = w + 2. * g.tile_width;
    let s = n + 2. * g.tile_height;
    let sample = |pixel: [usize; 2]| {
        let uv = g.uv(p.column + pixel[0] / 8, p.row + pixel[1] / 8);
        [
            uv.0 + (uv.1 - uv.0) * ((pixel[0] % 8) as f32 + 0.5) / 8.,
            uv.2 + (uv.3 - uv.2) * ((pixel[1] % 8) as f32 + 0.5) / 8.,
        ]
    };
    let [stone, edge, ink] = if down {
        [[12, 2], [1, 0], [0, 0]]
    } else {
        [[0, 0], [1, 2], [7, 3]]
    }
    .map(sample);
    let slab = |out: &mut SurfaceMeshData, b, lo, hi, uv| {
        let start = out.positions.len();
        rooms::slab(out, b, lo, hi, [1.; 4]);
        out.uvs[start..].fill(uv);
    };
    if down {
        slab(&mut mesh.textured, [w, e, n, s], -17. * k, -16. * k, ink);
        // Closed shaft cheeks with an open west approach to the first tread.
        for b in [[w, e, n, n + k], [w, e, s - k, s], [e - k, e, n + k, s - k]] {
            slab(&mut mesh.textured, b, -16. * k, k, edge);
        }
    } else {
        let uv = g.uv(p.ground % g.width, p.ground / g.width);
        for i in p.indices(g.width) {
            append_top(
                &mut mesh.textured,
                g.bounds(i % g.width, i / g.width).into(),
                0.,
                uv,
            );
        }
    }
    let left = if down { w } else { w + k };
    let right = e - k;
    for step in 0..6 {
        let x0 = left + (right - left) * step as f32 / 6.;
        let x1 = left + (right - left) * (step + 1) as f32 / 6.;
        let top = if down {
            -16. * k * (step + 1) as f32 / 6.
        } else {
            16. * k * (step + 1) as f32 / 6.
        };
        let base = if down { -17. * k } else { 0. };
        slab(&mut mesh.textured, [x0, x1, n + k, s - k], base, top, stone);
    }
    if !down {
        for b in [[w, e, n, n + k], [w, e, s - k, s]] {
            slab(&mut mesh.textured, b, 0., 16. * k, edge);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(down: bool, scale: f32) -> (VisualWorldFrame, GridGeometry, Placement) {
        use bevy::prelude::{IVec2, UVec2, Vec2};
        let g = GridGeometry {
            width: 12,
            height: 12,
            tile_width: 8. * scale,
            tile_height: 8. * scale,
            origin_x: 0.,
            origin_z: 0.,
        };
        let mut tiles = (0..144)
            .map(|i| VisualTile {
                column: (i % 12) as u32,
                row: (i / 12) as u32,
                texture: Default::default(),
                animation_frames: None,
                priority: false,
                source: VisualTileSource {
                    tileset_id: "lighthouse".into(),
                    metatile_id: 0x27,
                    subtile_column: (i % 12 % 4) as u8,
                    subtile_row: (i / 12 % 4) as u8,
                    tile_index: if (i % 12 + i / 12) % 2 == 0 {
                        0x2e
                    } else {
                        0x2f
                    },
                },
            })
            .collect::<Vec<_>>();
        let art = if down {
            [[0x27, 0x28], [0x37, 0x38]]
        } else {
            [[0x29, 0x2a], [0x39, 0x3a]]
        };
        for y in 0..2 {
            for x in 0..2 {
                tiles[(6 + y) * 12 + 6 + x].source = VisualTileSource {
                    tileset_id: "lighthouse".into(),
                    metatile_id: if down { 0x31 } else { 0x3a },
                    subtile_column: (x + 2) as u8,
                    subtile_row: (y + 2) as u8,
                    tile_index: art[y][x],
                };
            }
        }
        let refs = tiles.iter().collect::<Vec<_>>();
        let mut r = Resolver {
            map: "OlivineLighthouse1F",
            cells: &refs,
            g: &g,
            claimed: vec![false; 144],
            out: Vec::new(),
        };
        resolve_into(&mut r);
        assert_eq!(r.out.len(), 1);
        let p = r.out.remove(0);
        let frame = VisualWorldFrame {
            map_id: "OlivineLighthouse1F".into(),
            tiles,
            grid_size: UVec2::new(12, 12),
            tile_size: Vec2::splat(8. * scale),
            center: Vec2::new(-37., 29.),
            grid_origin: IVec2::new(-6, -10),
            ..Default::default()
        };
        (frame, g, p)
    }
    fn point(frame: &VisualWorldFrame, p: &Placement, x: f32, z: f32) -> bevy::prelude::Vec2 {
        use bevy::prelude::Vec2;
        frame.center
            + Vec2::new(
                (p.column as f32 + x / 8. - frame.grid_size.x as f32 * 0.5) * frame.tile_size.x,
                (frame.grid_size.y as f32 * 0.5 - p.row as f32 - z / 8.) * frame.tile_size.y,
            )
    }
    fn rendered_top(mesh: &TerrainMeshData, x: f32, z: f32) -> f32 {
        mesh.textured
            .indices
            .chunks_exact(3)
            .filter_map(|tri| {
                if tri
                    .iter()
                    .any(|i| mesh.textured.normals[*i as usize][1] < 0.99)
                {
                    return None;
                }
                let [a, b, c] =
                    [tri[0], tri[1], tri[2]].map(|i| mesh.textured.positions[i as usize]);
                let (e, f, d) = (
                    [b[0] - a[0], b[2] - a[2]],
                    [c[0] - a[0], c[2] - a[2]],
                    [x - a[0], z - a[2]],
                );
                let det = e[0] * f[1] - e[1] * f[0];
                if det.abs() < 1e-6 {
                    return None;
                }
                let u = (d[0] * f[1] - d[1] * f[0]) / det;
                let v = (e[0] * d[1] - e[1] * d[0]) / det;
                (u >= -1e-5 && v >= -1e-5 && u + v <= 1.00001)
                    .then_some(a[1] + u * (b[1] - a[1]) + v * (c[1] - a[1]))
            })
            .max_by(f32::total_cmp)
            .expect("sampled point has an actual rendered top")
    }
    fn check_contact(frame: &VisualWorldFrame, g: &GridGeometry, p: &Placement, down: bool) {
        let mut mesh = TerrainMeshData {
            footing_heights: vec![0.; g.width * g.height],
            ..Default::default()
        };
        append(&mut mesh, g, p, down);
        for step in 0..6 {
            let x = if down {
                15. * (step as f32 + 0.5) / 6.
            } else {
                1. + 14. * (step as f32 + 0.5) / 6.
            };
            let foot = point(frame, p, x, 8.);
            let resolved =
                crate::footing::resolved_footing_height(frame, foot, &mesh.footing_heights)
                    .unwrap();
            let actual = rendered_top(
                &mesh,
                (p.column as f32 + x / 8.) * g.tile_width,
                (p.row as f32 + 1.) * g.tile_height,
            );
            assert!(
                (resolved - actual).abs() < 0.001,
                "actual tread {down} step={step}: {resolved} vs {actual}"
            );
        }
        assert!(
            mesh.footing_heights.iter().all(|h| *h == 0.),
            "stored source support unchanged"
        );
    }
    #[test]
    fn lighthouse_stair_footing_matches_rendered_treads_and_rejects_unowned_sources() {
        for down in [false, true] {
            for scale in [0.5, 1., 2.] {
                let (mut frame, g, p) = fixture(down, scale);
                check_contact(&frame, &g, &p, down);
                frame.tiles.reverse();
                check_contact(&frame, &g, &p, down);
                let foot = point(&frame, &p, 8., 8.);
                let cell = crate::footing::tile_at_visual_point(&frame, foot).unwrap();
                let heights = vec![0.; 144];
                assert_eq!(
                    support_sample(&frame, cell, foot, &vec![9.; 144]),
                    None,
                    "foreign support plane"
                );
                for i in p.indices(g.width) {
                    let mut changed = frame.clone();
                    changed
                        .tiles
                        .iter_mut()
                        .find(|t| t.column as usize == i % 12 && t.row as usize == i / 12)
                        .unwrap()
                        .source
                        .tile_index ^= 1;
                    assert_eq!(
                        support_sample(&changed, cell, foot, &heights),
                        None,
                        "incomplete/altered source"
                    );
                }
                let mut changed = frame.clone();
                changed.map_id = "RadioTower1F".into();
                assert_eq!(support_sample(&changed, cell, foot, &heights), None);
                let mut changed = frame.clone();
                changed.tiles.push(changed.tiles[0].clone());
                assert_eq!(
                    support_sample(&changed, cell, foot, &heights),
                    None,
                    "duplicate grid"
                );
                let mut changed = frame.clone();
                changed.tiles.pop();
                assert_eq!(
                    support_sample(&changed, cell, foot, &heights),
                    None,
                    "clipped grid"
                );
                assert_eq!(
                    support_sample(&frame, cell, bevy::prelude::Vec2::splat(f32::NAN), &heights),
                    None
                );
                let mut changed = frame.clone();
                for t in &mut changed.tiles {
                    if t.source.metatile_id == 0x27 {
                        t.source.tile_index = 0xff;
                    }
                }
                assert_eq!(
                    support_sample(&changed, cell, foot, &heights),
                    None,
                    "no accepted native floor"
                );
            }
        }
    }
    #[test]
    fn lighthouse_stair_footing_keeps_walking_gait_and_authored_ledge_jump() {
        use bevy::prelude::Vec2;
        use crystal_render_api::{
            VisualActor, VisualActorId, VisualPlayerTraversal, VisualTraversalStep,
        };
        let (frame, _, p) = fixture(false, 1.);
        let heights = vec![0.; 144];
        let from = point(&frame, &p, 0.5, 8.);
        let to = point(&frame, &p, 8., 8.);
        let foot = from.lerp(to, 0.75);
        let actor = VisualActor {
            id: VisualActorId::Player,
            source_id: "chris".into(),
            model_palette: None,
            texture: Default::default(),
            center: foot,
            size: Vec2::new(8., 16.),
            flip_x: false,
            above_priority: false,
            facing: Some(Vec2::X),
        };
        let mut traversal = VisualPlayerTraversal {
            map_id: frame.map_id.clone(),
            step: Some(VisualTraversalStep {
                from: from - Vec2::Y * 8.,
                to: to - Vec2::Y * 8.,
                progress: 0.75,
                ledge_jump: false,
            }),
        };
        let (h, arc) =
            crate::footing::traversal_height(&frame, &actor, &heights, Some(&traversal)).unwrap();
        assert_eq!(
            h,
            crate::footing::resolved_footing_height(&frame, foot, &heights).unwrap()
        );
        assert_eq!(arc, 0., "stairs cannot invent a ledge hop");
        traversal.step.as_mut().unwrap().ledge_jump = true;
        assert!(
            crate::footing::traversal_height(&frame, &actor, &heights, Some(&traversal))
                .unwrap()
                .1
                > 0.,
            "authored ledge jump remains authoritative"
        );
    }
    #[test]
    fn external_lighthouse_stairs_match_all_fourteen_warps_and_keep_support() {
        let path = std::env::var_os("CRYSTAL_RENDER_TEST_PACK")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../content-packs/text-tui.crystalpack")
            });
        if !path.is_file() {
            return;
        }
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let pack = loaded.pack();
        let art = &pack.runtime_files()["data/tilesets/lighthouse_metatiles.bin"];
        let png = image::load_from_memory(&pack.runtime_files()["gfx/tilesets/lighthouse.png"])
            .unwrap()
            .to_luma8();
        let mut total = 0;
        for level in 1..=6 {
            let name = format!("OlivineLighthouse{level}F");
            let map = &pack.data().maps[&name];
            let (w, h) = (
                map.attributes.width as usize * 4,
                map.attributes.height as usize * 4,
            );
            let tiles = (0..w * h)
                .map(|i| {
                    let (x, y) = (i % w, i / w);
                    let b = map.blocks[y / 4 * (w / 4) + x / 4];
                    VisualTile {
                        column: x as u32,
                        row: y as u32,
                        texture: Default::default(),
                        priority: false,
                        animation_frames: None,
                        source: VisualTileSource {
                            tileset_id: "lighthouse".into(),
                            metatile_id: b,
                            subtile_column: (x % 4) as u8,
                            subtile_row: (y % 4) as u8,
                            tile_index: art[b as usize * 16 + y % 4 * 4 + x % 4] as u16,
                        },
                    }
                })
                .collect::<Vec<_>>();
            let refs = tiles.iter().collect::<Vec<_>>();
            let g = GridGeometry {
                width: w,
                height: h,
                tile_width: 8.,
                tile_height: 8.,
                origin_x: 0.,
                origin_z: 0.,
            };
            let placements = super::super::resolve(&name, &refs, &g, None);
            let stairs = placements
                .iter()
                .filter(|p| matches!(p.form, Form::LighthouseStair { .. }))
                .collect::<Vec<_>>();
            assert_eq!(stairs.len(), [1, 2, 3, 4, 3, 1][level - 1]);
            total += stairs.len();
            for warp in &map.events.warps {
                let i = warp.y as usize * 2 * w + warp.x as usize * 2;
                assert_eq!(
                    stairs.iter().any(|p| p.indices(w).any(|j| i == j)),
                    local(&tiles[i].source).is_some(),
                    "{name}: floor/pit/carpet isn't a stair"
                );
            }
            for p in stairs {
                let Form::LighthouseStair { down } = p.form else {
                    unreachable!()
                };
                let frame = VisualWorldFrame {
                    map_id: name.clone().into(),
                    tiles: tiles.clone(),
                    grid_size: bevy::prelude::UVec2::new(w as u32, h as u32),
                    tile_size: bevy::prelude::Vec2::splat(8.),
                    center: bevy::prelude::Vec2::new(-37., 29.),
                    ..Default::default()
                };
                check_contact(&frame, &g, p, down);
                let mut mesh = TerrainMeshData {
                    footing_heights: vec![4.5; w * h],
                    ..Default::default()
                };
                append(&mut mesh, &g, p, down);
                assert_eq!(mesh.footing_heights, vec![4.5; w * h]);
                assert!(mesh.solid.positions.is_empty());
                assert!(!mesh.textured.positions.is_empty());
                let samples = if down {
                    [[12, 2], [1, 0], [0, 0]]
                } else {
                    [[0, 0], [1, 2], [7, 3]]
                };
                for (pixel, value) in samples.into_iter().zip([170, 85, 0]) {
                    let s = &tiles[(p.row + pixel[1] / 8) * w + p.column + pixel[0] / 8].source;
                    assert_eq!(
                        png.get_pixel(
                            (s.tile_index as u32 % 16) * 8 + (pixel[0] % 8) as u32,
                            (s.tile_index as u32 / 16) * 8 + (pixel[1] % 8) as u32
                        )[0],
                        value
                    );
                }
                assert_eq!(mesh.textured.positions.iter().any(|p| p[1] < 0.), down);
                for i in p.indices(w) {
                    let mut changed = tiles.clone();
                    changed[i].source.tile_index ^= 1;
                    assert!(
                        !super::super::resolve(
                            &name,
                            &changed.iter().collect::<Vec<_>>(),
                            &g,
                            None
                        )
                        .iter()
                        .any(|q| matches!(
                            q.form,
                            Form::LighthouseStair { .. }
                        ) && q.column == p.column
                            && q.row == p.row)
                    );
                }
            }
        }
        assert_eq!(total, 14);
    }
}
