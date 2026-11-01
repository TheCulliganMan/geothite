//! Source-painted Colosseum/Time Capsule/Trade Center finishes on their existing room kit.
//! Admission remains the complete native object/cell fingerprint, never a name
//! alone. Source ownership, plot geometry and gameplay support are independent.
use super::*;

fn native_cell(cells: &[&VisualTile], g: &GridGeometry, p: &Placement, detail: Detail) -> bool {
    if p.width != 1 || p.height != 1 || p.column >= g.width || p.row >= g.height {
        return false;
    }
    let source = &cells[p.row * g.width + p.column].source;
    ["Colosseum", "TimeCapsule"]
        .into_iter()
        .any(|map| match (detail, cell_detail(map, source)) {
            (Detail::Wall(_) | Detail::WallSegment { .. }, Some(Detail::Wall(_))) => true,
            (Detail::Surface(Surface::Ceramic), Some(Detail::Surface(Surface::Ceramic))) => true,
            (Detail::Threshold, Some(Detail::Threshold)) => true,
            _ => false,
        })
}
fn floor(cells: &[&VisualTile]) -> Option<usize> {
    cells.iter().position(|tile| {
        let s = &tile.source;
        s.tileset_id.as_ref() == "gate"
            && s.metatile_id == 0x17
            && s.tile_index == 0x01
            && matches!(
                cell_detail("Colosseum", s),
                Some(Detail::Surface(Surface::Ceramic))
            )
    })
}
fn underlay(mesh: &mut TerrainMeshData, g: &GridGeometry, p: &Placement, sample: usize) {
    for i in p.indices(g.width) {
        append_top(
            &mut mesh.textured,
            g.bounds(i % g.width, i / g.width).into(),
            0.,
            g.uv(sample % g.width, sample / g.width),
        );
    }
}
fn painted(target: &mut SurfaceMeshData, part: SurfaceMeshData, uv: [f32; 2]) {
    let first = target.positions.len() as u32;
    for (position, normal) in part.positions.into_iter().zip(part.normals) {
        target.positions.push(position);
        target.normals.push(normal);
        target.uvs.push(uv);
        target
            .colors
            .push(crate::interior_models::authored_face_color(
                [1.; 4],
                Vec3::from_array(normal),
            ));
    }
    target
        .indices
        .extend(part.indices.into_iter().map(|i| i + first));
}
fn slab_painted(out: &mut SurfaceMeshData, b: [f32; 4], lo: f32, hi: f32, uv: [f32; 2]) {
    let mut part = SurfaceMeshData::default();
    slab(&mut part, b, lo, hi, [1.; 4]);
    painted(out, part, uv);
}
fn octagon_painted(out: &mut SurfaceMeshData, b: [f32; 4], lo: f32, hi: f32, uv: [f32; 2]) {
    let mut part = SurfaceMeshData::default();
    octagon(&mut part, b, lo, hi, [1.; 4]);
    painted(out, part, uv);
}

pub(super) fn append(
    mesh: &mut TerrainMeshData,
    g: &GridGeometry,
    p: &Placement,
    cells: &[&VisualTile],
    detail: Detail,
    style: FloorStyle,
) -> bool {
    if style != FloorStyle::LinkRoom
        || g.width == 0
        || cells.len() != g.width * g.height
        || p.column + p.width > g.width
        || p.row + p.height > g.height
    {
        return false;
    }
    let Some(ground) = floor(cells) else {
        return false;
    };
    let (w, _, n, _) = g.bounds(p.column, p.row);
    let k = g.tile_height / SOURCE_TILE_HEIGHT;
    let e = w + p.width as f32 * g.tile_width;
    let s = n + p.height as f32 * g.tile_height;
    if let Detail::LinkPlatform { trade } = detail {
        let Some(object) = OBJECTS
            .iter()
            .find(|o| matches!(o.detail,Detail::LinkPlatform{trade:t} if t==trade))
        else {
            return false;
        };
        if p.width != object.width
            || p.height != object.height
            || !matches_object(object.map, cells, g, p.column, p.row, object)
        {
            return false;
        }
        let console = room_model(if trade {
            Asset::LinkTradeConsole
        } else {
            Asset::LinkBattleConsole
        });
        if !console.has_native_link_materials()
            || (trade && !room_model(Asset::LinkRoomReceiver).has_native_link_materials())
        {
            return false;
        }
        underlay(mesh, g, p, ground);
        // Original octagon positions and layer datums; only its own native
        // gray outline/beige fill replace the former synthetic steel colors.
        let edge = source_texel_uv(g, p, [32, 0]);
        let fill = source_texel_uv(g, p, [32, 24]);
        for (bounds, low, high, uv) in [
            ([w + k, e - k, n + k, s - k], -k, 0.005 * k, edge),
            (
                [w + 3. * k, e - 3. * k, n + 3. * k, s - 3. * k],
                -0.4 * k,
                0.008 * k,
                fill,
            ),
            (
                [w + 7. * k, e - 7. * k, n + 7. * k, s - 7. * k],
                -0.2 * k,
                0.012 * k,
                edge,
            ),
            (
                [w + 9. * k, e - 9. * k, n + 9. * k, s - 9. * k],
                -0.15 * k,
                0.016 * k,
                fill,
            ),
        ] {
            octagon_painted(&mut mesh.textured, bounds, low, high, uv);
        }
        let [body, light, ink] = if trade {
            [[35, 33], [32, 32], [33, 32]]
        } else {
            [[40, 33], [41, 32], [40, 32]]
        }
        .map(|pixel| source_texel_uv(g, p, pixel));
        let swatches = [body, light, ink, ink, light, body, light];
        let (cx, cz) = ((w + e) * 0.5, n + 5. * g.tile_height);
        assert!(console.append_native_link(
            &mut mesh.textured,
            [cx - 16. * k, cx + 16. * k, cz - 12. * k, cz + 4. * k],
            0.02 * k,
            if trade { 13. * k } else { 20. * k },
            swatches
        ));
        for (seat_x, source_x) in [(cx - 24. * k, 16), (cx + 24. * k, 64)] {
            sampled_room_asset(
                mesh,
                g,
                p,
                Asset::LinkRoundStool,
                [source_x, 32],
                [seat_x - 6. * k, seat_x + 6. * k, cz - 6. * k, cz + 6. * k],
                0.02 * k,
                6. * k,
            );
        }
        if trade {
            let blue = source_texel_uv(g, p, [40, 8]);
            let paper = source_texel_uv(g, p, [41, 9]);
            assert!(room_model(Asset::LinkRoomReceiver).append_native_link(
                &mut mesh.textured,
                [cx - 8. * k, cx + 8. * k, n + 8. * k, n + 18. * k],
                0.02 * k,
                17. * k,
                [blue, paper, blue, blue, paper, blue, paper]
            ));
        }
        return true;
    }
    if !native_cell(cells, g, p, detail) {
        return false;
    }
    match detail {
        Detail::Surface(Surface::Ceramic) | Detail::Threshold => {
            // Native checker, waist strip and red entry retain their exact
            // drawing rather than acquiring generic ceramic/grating colors.
            append_top(&mut mesh.textured, [w, e, n, s], 0., g.uv(p.column, p.row));
        }
        Detail::Wall(_) | Detail::WallSegment { .. } => {
            let source = &cells[p.row * g.width + p.column].source;
            let Some(row) = p.row.checked_sub(usize::from(source.subtile_row)) else {
                return false;
            };
            if source.metatile_id != 0x16 || source.subtile_row > 2 || row + 2 >= g.height {
                return false;
            }
            let blue = &cells[row * g.width + p.column].source;
            let gray = &cells[(row + 1) * g.width + p.column].source;
            let waist = &cells[(row + 2) * g.width + p.column].source;
            if blue.metatile_id != 0x16
                || blue.subtile_row != 0
                || gray.metatile_id != 0x16
                || gray.subtile_row != 1
                || !matches!(cell_detail("Colosseum", blue), Some(Detail::Wall(_)))
                || !matches!(cell_detail("Colosseum", gray), Some(Detail::Wall(_)))
                || waist.metatile_id != 0x16
                || waist.subtile_row != 2
                || !matches!(cell_detail("Colosseum", waist), Some(Detail::Wall(_)))
            {
                return false;
            }
            let mut donor = p.clone();
            donor.row = row;
            let body = source_texel_uv(g, &donor, [0, 1]);
            let cap = source_texel_uv(g, &donor, [0, 0]);
            let highlight = source_texel_uv(g, &donor, [4, 2]);
            donor.column = ground % g.width;
            donor.row = ground / g.width;
            let skirt = source_texel_uv(g, &donor, [7, 0]);
            underlay(mesh, g, p, ground);
            let out = &mut mesh.textured;
            slab_painted(out, [w, e, n, s], 0., 14. * k, body);
            slab_painted(out, [w, e, n, s], 14. * k, 16. * k, cap);
            let open = if let Detail::WallSegment { open, .. } = detail {
                open
            } else {
                [true; 4]
            };
            for (j, bounds) in [
                [w, e, n, n + 0.28 * k],
                [e - 0.28 * k, e, n, s],
                [w, e, s - 0.28 * k, s],
                [w, w + 0.28 * k, n, s],
            ]
            .into_iter()
            .enumerate()
            {
                if open[j] {
                    slab_painted(out, bounds, 0., 1.7 * k, skirt);
                    slab_painted(out, bounds, 10. * k, 11. * k, highlight);
                }
            }
        }
        _ => return false,
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn positive_triangles(mesh: &TerrainMeshData) -> BTreeMap<Vec<i64>, usize> {
        let mut result = BTreeMap::new();
        for surface in [&mesh.textured, &mesh.solid] {
            for triangle in surface.indices.chunks_exact(3) {
                let points = triangle
                    .iter()
                    .map(|i| surface.positions[*i as usize])
                    .collect::<Vec<_>>();
                if !points.iter().any(|p| p[1] > 0.) {
                    continue;
                }
                let mut key = points
                    .into_iter()
                    .map(|p| p.map(|v| (v * 10000.).round() as i64))
                    .collect::<Vec<_>>();
                key.sort();
                *result
                    .entry(key.into_iter().flatten().collect())
                    .or_default() += 1;
            }
        }
        result
    }
    #[test]
    fn external_link_rooms_keep_native_palette_and_existing_upright_geometry() {
        let path = std::env::var_os("CRYSTAL_RENDER_TEST_PACK")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../content-packs/text-tui.crystalpack")
            });
        assert!(
            path.is_file(),
            "explicit local content pack required for native link-room proof"
        );
        let loaded = crystal_assets::read_loaded_verified_compiled_game_pack(&path).unwrap();
        let pack = loaded.pack();
        let art = &pack.runtime_files()["data/tilesets/gate_metatiles.bin"];
        let image = image::load_from_memory(&pack.runtime_files()["gfx/tilesets/gate.png"])
            .unwrap()
            .to_luma8();
        let palettes = &pack.data().tilesets["gate"].palette_map;
        for (map, trade) in [
            ("Colosseum", false),
            ("TimeCapsule", true),
            ("TradeCenter", true),
        ] {
            let data = &pack.data().maps[map];
            let (width, height) = (
                data.attributes.width as usize * 4,
                data.attributes.height as usize * 4,
            );
            let tiles = (0..width * height)
                .map(|i| {
                    let (x, y) = (i % width, i / width);
                    let b = data.blocks[y / 4 * (width / 4) + x / 4];
                    VisualTile {
                        column: x as u32,
                        row: y as u32,
                        texture: Default::default(),
                        priority: false,
                        animation_frames: None,
                        source: VisualTileSource {
                            tileset_id: "gate".into(),
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
                width,
                height,
                tile_width: 8.,
                tile_height: 8.,
                origin_x: 0.,
                origin_z: 0.,
            };
            let mut resolver = Resolver {
                map,
                cells: &refs,
                g: &g,
                claimed: vec![false; tiles.len()],
                out: Vec::new(),
            };
            super::super::resolve_into(&mut resolver);
            assert!(
                resolver.claimed.iter().all(|v| *v),
                "complete link-room source coverage"
            );
            let platform = resolver
                .out
                .iter()
                .find(|p| matches!(p.form, Form::Room(Detail::LinkPlatform { .. }, _)))
                .unwrap();
            let pigment = |p: &Placement, pixel: [usize; 2]| {
                let raw = refs[(p.row + pixel[1] / 8) * width + p.column + pixel[0] / 8]
                    .source
                    .tile_index as usize;
                let bank = palettes[raw];
                let count = (image.width() / 8) * (image.height() / 8);
                let tile = if bank & 8 != 0 {
                    (raw & 127) + count as usize / 2
                } else {
                    raw
                };
                (
                    bank & 7,
                    image.get_pixel(
                        (tile % 16 * 8 + pixel[0] % 8) as u32,
                        (tile / 16 * 8 + pixel[1] % 8) as u32,
                    )[0],
                )
            };
            let green = if trade {
                [[35, 33], [32, 32], [33, 32]]
            } else {
                [[40, 33], [41, 32], [40, 32]]
            };
            for (pixel, gray) in green.into_iter().zip([85, 170, 0]) {
                assert_eq!(
                    pigment(platform, pixel),
                    (2, gray),
                    "own green equipment pigment"
                );
            }
            for (pixel, gray) in [([32, 0], 85), ([32, 24], 255)] {
                assert_eq!(
                    pigment(platform, pixel),
                    (0, gray),
                    "gray/beige octagon drawing"
                );
            }
            if trade {
                assert_eq!(pigment(platform, [40, 8]), (3, 85));
                assert_eq!(pigment(platform, [41, 9]), (3, 255));
            }
            let mut native = TerrainMeshData {
                footing_heights: vec![3.25; tiles.len()],
                ..Default::default()
            };
            let mut old = TerrainMeshData {
                footing_heights: native.footing_heights.clone(),
                ..Default::default()
            };
            let mut walls = 0;
            let mut threshold = 0;
            for p in &resolver.out {
                let Form::Room(detail, style) = p.form else {
                    panic!("expected exact special-room ownership")
                };
                assert_eq!(style, FloorStyle::LinkRoom);
                assert!(
                    append(&mut native, &g, p, &refs, detail, style),
                    "all accepted native link details paint"
                );
                super::super::append_with_floor(
                    &mut old,
                    &g,
                    p,
                    &refs,
                    detail,
                    FloorStyle::Default,
                );
                if matches!(detail, Detail::Wall(_) | Detail::WallSegment { .. }) {
                    walls += 1;
                    let mut donor = p.clone();
                    donor.row -= usize::from(refs[p.row * width + p.column].source.subtile_row);
                    for (pixel, gray) in [([0, 1], 85), ([0, 0], 170), ([4, 2], 255)] {
                        assert_eq!(pigment(&donor, pixel), (3, gray));
                    }
                }
                if matches!(detail, Detail::Threshold) {
                    threshold += 1;
                    assert_eq!(pigment(p, [0, 1]).0, 1, "red entry is not gray steel");
                }
            }
            assert!(walls > 0 && threshold > 0);
            assert!(native.solid.indices.is_empty());
            assert_eq!(native.footing_heights, vec![3.25; tiles.len()]);
            // Exercise the real final pass on the entire packed room, not
            // just isolated plain-floor blocks: the octagon corner blocks
            // also contain diamond ground beside the entrance.
            let before_finish = native.clone();
            assert_eq!(
                crate::mesh::modeled_interiors::finish_surfaces_with_profiles(
                    &mut native,
                    map,
                    &refs,
                    &g,
                    [0, 0],
                    None
                ),
                0,
                "the complete native link kit already owns its floor paint"
            );
            assert_eq!(
                native, before_finish,
                "final pass preserves the entire native kit"
            );
            // Only the entry's old raised grating ribs are removed. Other
            // platform, equipment, stool and wall triangles remain identical.
            let mut old_without_entry = TerrainMeshData::default();
            let mut new_without_entry = TerrainMeshData::default();
            for p in &resolver.out {
                let Form::Room(detail, style) = p.form else {
                    unreachable!()
                };
                if matches!(detail, Detail::Threshold) {
                    continue;
                }
                super::super::append_with_floor(
                    &mut old_without_entry,
                    &g,
                    p,
                    &refs,
                    detail,
                    FloorStyle::Default,
                );
                assert!(append(&mut new_without_entry, &g, p, &refs, detail, style));
            }
            assert_eq!(
                positive_triangles(&old_without_entry),
                positive_triangles(&new_without_entry)
            );
            assert!(
                native
                    .textured
                    .colors
                    .iter()
                    .all(|c| c[0] == c[1] && c[1] == c[2] && c[3] == 1.),
                "only native atlas supplies hue"
            );
            let Form::Room(detail, style) = platform.form else {
                unreachable!()
            };
            for i in platform.indices(width) {
                let mut changed = tiles.clone();
                changed[i].source.tile_index ^= 1;
                let changed = changed.iter().collect::<Vec<_>>();
                let mut out = TerrainMeshData::default();
                assert!(!append(&mut out, &g, platform, &changed, detail, style));
                assert!(
                    out.textured.indices.is_empty() && out.solid.indices.is_empty(),
                    "altered complete plot retains fallback"
                );
            }
            let mut out = TerrainMeshData::default();
            assert!(!append(
                &mut out,
                &g,
                platform,
                &refs,
                detail,
                FloorStyle::Default
            ));
            assert!(out.textured.indices.is_empty());
        }
    }
}
