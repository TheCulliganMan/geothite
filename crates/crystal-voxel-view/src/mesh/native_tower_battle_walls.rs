//! Native gray panels, blue skirting and the opponent's staging entrance.
use super::*;
use bevy::prelude::{IVec2, UVec2};

const MAP: &str = "BattleTowerBattleRoom";

fn panel(map: &str, cells: &[&VisualTile], g: &GridGeometry, p: &Placement, door: bool) -> bool {
    p.width == 2
        && p.height == 2
        && OBJECTS.iter().any(|o| {
            matches!(
                (door, o.detail),
                (false, Detail::Asset(Asset::TowerEmblemPanel)) | (true, Detail::TowerStagingDoor)
            ) && matches_object(map, cells, g, p.column, p.row, o)
        })
}

fn gray(cells: &[&VisualTile], g: &GridGeometry) -> Option<[f32; 2]> {
    for o in OBJECTS
        .iter()
        .filter(|o| matches!(o.detail, Detail::Asset(Asset::TowerEmblemPanel)))
    {
        for row in 0..g.height.saturating_sub(1) {
            for column in 0..g.width.saturating_sub(1) {
                if matches_object(MAP, cells, g, column, row, o) {
                    // Tile04's lower-right field is source shade2 in GRAY.
                    let uv = g.uv(column, row);
                    return Some([
                        uv.0 + (uv.1 - uv.0) * 7.5 / 8.,
                        uv.2 + (uv.3 - uv.2) * 7.5 / 8.,
                    ]);
                }
            }
        }
    }
    None
}

fn body(out: &mut SurfaceMeshData, b: [f32; 4], height: f32, uv: [f32; 2]) {
    let first = out.positions.len();
    slab(out, b, 0., height, [1.; 4]);
    out.uvs[first..].fill(uv);
    for i in first..out.positions.len() {
        out.colors[i] =
            crate::interior_models::authored_face_color([1.; 4], Vec3::from_array(out.normals[i]));
    }
}

fn face(
    out: &mut SurfaceMeshData,
    g: &GridGeometry,
    source: [usize; 2],
    size: [usize; 2],
    origin: Vec3,
    along: Vec3,
    dimensions: [f32; 2],
) {
    let normal = along.cross(Vec3::Y);
    for y in 0..size[1] {
        for x in 0..size[0] {
            let uv = g.uv(source[0] + x, source[1] + y);
            let left = dimensions[0] * x as f32 / size[0] as f32;
            let right = dimensions[0] * (x + 1) as f32 / size[0] as f32;
            let top = dimensions[1] * (size[1] - y) as f32 / size[1] as f32;
            let bottom = dimensions[1] * (size[1] - y - 1) as f32 / size[1] as f32;
            let at = |s: f32, h: f32| (origin + along * s + Vec3::Y * h).to_array();
            append_quad(
                out,
                [
                    at(right, bottom),
                    at(right, top),
                    at(left, top),
                    at(left, bottom),
                ],
                normal.to_array(),
                [[uv.1, uv.3], [uv.1, uv.2], [uv.0, uv.2], [uv.0, uv.3]],
                crate::interior_models::authored_face_color([1.; 4], normal),
            );
        }
    }
}

pub(super) fn append(
    mesh: &mut TerrainMeshData,
    g: &GridGeometry,
    p: &Placement,
    cells: &[&VisualTile],
    detail: Detail,
    style: FloorStyle,
) -> bool {
    if style != FloorStyle::TowerArena
        || cells.len() != g.width * g.height
        || p.column + p.width > g.width
        || p.row + p.height > g.height
    {
        return false;
    }
    let is_panel =
        matches!(detail, Detail::Asset(Asset::TowerEmblemPanel)) && panel(MAP, cells, g, p, false);
    let door = matches!(detail, Detail::TowerStagingDoor) && panel(MAP, cells, g, p, true);
    let wall = matches!(detail, Detail::Wall(_) | Detail::WallSegment { .. })
        && p.width == 1
        && p.height == 1
        && matches!(
            cell_detail(MAP, &cells[p.row * g.width + p.column].source),
            Some(Detail::Wall(_))
        );
    if !is_panel && !door && !wall {
        return false;
    }
    let Some(paint) = gray(cells, g) else {
        return false;
    };
    let source = &cells[p.row * g.width + p.column].source;
    if wall && source.tile_index == 0x25 && source.metatile_id == 0x10 && source.subtile_column < 2
    {
        // One original floor surface, with no underlay or lip across the
        // opponent's approach. Its blue border is part of the walkable tile.
        append_top(
            &mut mesh.textured,
            g.bounds(p.column, p.row).into(),
            0.,
            g.uv(p.column, p.row),
        );
        return true;
    }
    if !native_tower_floor::underlay(mesh, g, p, cells) {
        return false;
    }
    let (w, _, n, _) = g.bounds(p.column, p.row);
    let e = w + p.width as f32 * g.tile_width;
    let s = n + p.height as f32 * g.tile_height;
    let k = g.tile_height / 8.;
    let first = mesh.textured.positions.len();
    if is_panel {
        body(&mut mesh.textured, [w, e, n, s - 0.02 * k], 32. * k, paint);
        face(
            &mut mesh.textured,
            g,
            [p.column, p.row],
            [2, 2],
            Vec3::new(w, 8. * k, s),
            Vec3::X,
            [e - w, 16. * k],
        );
    } else if door {
        // The original opponent stands at the DOOR tile center. Leaves stay
        // behind that center; the front remains clear below the 26px lintel.
        for [a, b] in [[w, w + k], [e - k, e]] {
            body(&mut mesh.textured, [a, b, n, s], 32. * k, paint);
        }
        let start = mesh.textured.positions.len();
        body(&mut mesh.textured, [w + k, e - k, n, s], 6. * k, paint);
        for point in &mut mesh.textured.positions[start..] {
            point[1] += 26. * k;
        }
        face(
            &mut mesh.textured,
            g,
            [p.column, p.row],
            [2, 2],
            Vec3::new(w + k, 0., n + 0.02 * k),
            Vec3::X,
            [e - w - 2. * k, 24. * k],
        );
    } else {
        let source = &cells[p.row * g.width + p.column].source;
        if source.tile_index == 0x25 {
            // This projected base row belongs on the wall behind it.
            // Its source quadrant is walkable; leave the whole strip free.
            face(
                &mut mesh.textured,
                g,
                [p.column, p.row],
                [1, 1],
                Vec3::new(w, 0., n),
                Vec3::X,
                [e - w, 8. * k],
            );
        } else {
            body(&mut mesh.textured, [w, e, n, s], 32. * k, paint);
            append_top(
                &mut mesh.textured,
                [w, e, n, s],
                32.002 * k,
                g.uv(p.column, p.row),
            );
        }
    }
    mesh.textured
        .cutaway_ranges
        .push(first..mesh.textured.positions.len());
    true
}

fn index(frame: &VisualWorldFrame, g: &GridGeometry, x: i32, y: i32) -> Option<usize> {
    let x = x - frame.grid_origin.x;
    let y = y - frame.grid_origin.y;
    (x >= 0 && y >= 0 && (x as usize) < g.width && (y as usize) < g.height)
        .then(|| y as usize * g.width + x as usize)
}

pub(super) fn perimeter(
    mesh: &mut TerrainMeshData,
    frame: &VisualWorldFrame,
    cells: &[&VisualTile],
    g: &GridGeometry,
) {
    if frame.map_id.as_ref() != MAP
        || frame.source_map_size_core_tiles != Some(UVec2::new(8, 8))
        || cells.len() != g.width * g.height
        || frame.boundary_warps.len() != 2
        || ![IVec2::new(3, 7), IVec2::new(4, 7)]
            .iter()
            .all(|p| frame.boundary_warps.contains(p))
    {
        return;
    }
    // Compact identity of the complete reviewed rear drawing, including its
    // approach and corner rows. Never infer an enclosure behind altered art.
    let mut band = Vec::new();
    for y in 0..4 {
        for x in 0..16 {
            let Some(i) = index(frame, g, x, y) else {
                return;
            };
            if cells[i].source.tileset_id.as_ref() != "battle_tower_inside" {
                return;
            }
            band.push(&cells[i].source);
        }
    }
    if identity_hash(band) != 0x20fa2304eae85869 {
        return;
    }
    let Some(paint) = gray(cells, g) else {
        return;
    };
    let source = index(frame, g, 4, 0).unwrap();
    let base = index(frame, g, 4, 2).unwrap();
    let source = [source % g.width, source / g.width];
    let base = [base % g.width, base / g.width];
    let k = g.tile_height / 8.;
    let west = g.origin_x - frame.grid_origin.x as f32 * g.tile_width;
    let east = west + 16. * g.tile_width;
    let north = g.origin_z - frame.grid_origin.y as f32 * g.tile_height;
    let south = north + 16. * g.tile_height;
    for (b, origin, along, plane) in [
        (
            [west - 3. * k, west, north, south + 3. * k],
            Vec3::new(west, 0., south),
            Vec3::NEG_Z,
            [1., 0., west],
        ),
        (
            [east, east + 3. * k, north, south + 3. * k],
            Vec3::new(east, 0., north),
            Vec3::Z,
            [-1., 0., -east],
        ),
        (
            [west, east, south, south + 3. * k],
            Vec3::new(east, 0., south),
            Vec3::NEG_X,
            [0., -1., -south],
        ),
    ] {
        // South segments intersecting the actual two source warps are omitted.
        // The outside faces still close continuously around both corners.
        for section in 0..8 {
            if plane[1] != 0. && matches!(section, 3 | 4) {
                continue;
            }
            let start = mesh.textured.positions.len();
            let step = if along.z != 0. {
                2. * g.tile_height
            } else {
                2. * g.tile_width
            };
            let at = origin + along * (section as f32 * step);
            let mut bounds = b;
            if along.z != 0. {
                let end = at.z + along.z * step;
                bounds[2] = at.z.min(end);
                bounds[3] = at.z.max(end);
            } else {
                let end = at.x + along.x * step;
                bounds[0] = at.x.min(end);
                bounds[1] = at.x.max(end);
            }
            body(&mut mesh.textured, bounds, 32. * k, paint);
            // Keep the native ornament in front of the body's inward face.
            // Coplanar source quads flicker when viewed along either side.
            let face_at = at + along.cross(Vec3::Y) * (0.02 * k);
            face(
                &mut mesh.textured,
                g,
                base,
                [2, 1],
                face_at,
                along,
                [step, 8. * k],
            );
            face(
                &mut mesh.textured,
                g,
                source,
                [2, 2],
                face_at + Vec3::Y * 8. * k,
                along,
                [step, 16. * k],
            );
            let range = start..mesh.textured.positions.len();
            mesh.textured.cutaway_ranges.push(range.clone());
            mesh.camera_wall_textured_ranges.push((range, plane));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clear(mesh: &SurfaceMeshData, point: Vec3) -> bool {
        mesh.indices.chunks_exact(3).all(|t| {
            let [a, b, c] =
                [t[0], t[1], t[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
            let normal = (b - a).cross(c - a);
            if normal.length() < 0.0001 || (point - a).dot(normal).abs() / normal.length() > 0.001 {
                return true;
            }
            ![(a, b), (b, c), (c, a)]
                .iter()
                .all(|&(u, v)| (v - u).cross(point - u).dot(normal) >= -0.001)
        })
    }

    #[test]
    fn external_tower_battle_walls_keep_gray_panels_blue_base_and_both_entrances_clear() {
        let Some(path) = std::env::var_os("CRYSTAL_RENDER_TEST_PACK") else {
            return;
        };
        let pack = crystal_assets::read_loaded_verified_compiled_game_pack(path).unwrap();
        let room = &pack.pack().data().maps[MAP];
        let layout =
            &pack.pack().runtime_files()["data/tilesets/battle_tower_inside_metatiles.bin"];
        let (w, h) = (
            room.attributes.width as usize * 4,
            room.attributes.height as usize * 4,
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
                let block = room.blocks[y / 4 * (w / 4) + x / 4];
                VisualTile {
                    column: x as u32,
                    row: y as u32,
                    texture: Default::default(),
                    animation_frames: None,
                    priority: false,
                    source: VisualTileSource {
                        tileset_id: "battle_tower_inside".into(),
                        metatile_id: block,
                        subtile_column: (x % 4) as u8,
                        subtile_row: (y % 4) as u8,
                        tile_index: layout[block as usize * 16 + y % 4 * 4 + x % 4] as u16,
                    },
                }
            })
            .collect::<Vec<_>>();
        let refs = tiles.iter().collect::<Vec<_>>();
        let mut r = Resolver {
            map: MAP,
            cells: &refs,
            g: &g,
            claimed: vec![false; w * h],
            out: vec![],
        };
        resolve_into(&mut r);
        assert!(r.claimed.iter().all(|&v| v));
        let mut whole = TerrainMeshData {
            footing_heights: vec![2.75; w * h],
            ..Default::default()
        };
        let mut counts = [0; 3];
        for p in &r.out {
            let Form::Room(detail, style) = p.form else {
                unreachable!()
            };
            let mut part = TerrainMeshData {
                footing_heights: vec![2.75; w * h],
                ..Default::default()
            };
            if !append(&mut part, &g, p, &refs, detail, style) {
                continue;
            }
            let kind = match detail {
                Detail::Asset(Asset::TowerEmblemPanel) => 0,
                Detail::TowerStagingDoor => 1,
                _ => 2,
            };
            counts[kind] += 1;
            let source = &refs[p.row * w + p.column].source;
            if kind == 2
                && source.metatile_id == 0x10
                && source.subtile_row == 2
                && source.subtile_column < 2
            {
                assert_eq!(
                    part.textured.indices.len(),
                    6,
                    "one staging-approach floor surface"
                );
                assert!(
                    part.textured.positions.iter().all(|p| p[1] == 0.),
                    "no lip beneath the opponent"
                );
            }
            assert_eq!(part.footing_heights, vec![2.75; w * h]);
            assert!(part.solid.indices.is_empty());
            assert!(
                part.textured
                    .positions
                    .iter()
                    .all(|p| p[1] >= 0. && p[1] <= 32.01)
            );
            for t in part.textured.indices.chunks_exact(3) {
                let [a, b, c] = [t[0], t[1], t[2]]
                    .map(|i| Vec3::from_array(part.textured.positions[i as usize]));
                let normal = Vec3::from_array(part.textured.normals[t[0] as usize]);
                assert!(
                    (b - a).cross(c - a).normalize().dot(normal) > 0.99,
                    "outward wall/door/trim winding"
                );
            }
            for bad in p.indices(w) {
                let mut changed = tiles.clone();
                changed[bad].source.tile_index ^= 0x80;
                assert!(!append(
                    &mut TerrainMeshData::default(),
                    &g,
                    p,
                    &changed.iter().collect::<Vec<_>>(),
                    detail,
                    style
                ));
            }
            assert!(!append(
                &mut TerrainMeshData::default(),
                &g,
                p,
                &refs,
                detail,
                FloorStyle::Default
            ));
            append(&mut whole, &g, p, &refs, detail, style);
        }
        assert_eq!(counts, [7, 1, 20]);
        // The FLOOR strip under the projected blue base is one complete walk
        // lane. A wall across it would hide/clamp both staging actors and player.
        for x in 2..14 {
            for y in [1., 8., 16., 23.] {
                assert!(
                    clear(&whole.textured, Vec3::new((x as f32 + 0.5) * 8., y, 20.)),
                    "clear rear FLOOR at cell{x}"
                );
            }
        }
        for z in [4., 8., 12., 16., 20.] {
            for y in [1., 8., 16., 23.] {
                assert!(
                    clear(&whole.textured, Vec3::new(72., y, z)),
                    "opponent staging entrance stays clear"
                );
            }
        }
        for x in [24., 40., 56., 88., 104.] {
            assert!(
                !clear(&whole.textured, Vec3::new(x, 20., 16.)),
                "native gray panel face"
            );
            assert!(
                !clear(&whole.textured, Vec3::new(x, 4., 16.)),
                "native blue skirting"
            );
            assert!(
                !clear(&whole.textured, Vec3::new(x, 28., 15.98)),
                "connected full-height body"
            );
        }
        let palette = &pack.pack().data().tilesets["battle_tower_inside"].palette_map;
        let uv = gray(&refs, &g).unwrap();
        let donor = (uv[1] * h as f32).floor() as usize * w + (uv[0] * w as f32).floor() as usize;
        assert_eq!(
            palette[refs[donor].source.tile_index as usize] & 7,
            0,
            "body paint uses source GRAY"
        );
        let image = image::load_from_memory(
            &pack.pack().runtime_files()["gfx/tilesets/battle_tower_inside.png"],
        )
        .unwrap()
        .to_luma8();
        let tile = refs[donor].source.tile_index as u32;
        assert_eq!(
            image.get_pixel(tile % 16 * 8 + 7, tile / 16 * 8 + 7).0[0],
            170,
            "native gray field, not black border or pale cap"
        );

        let mut frame = crate::mesh::tests::frame(
            w as u32,
            h as u32,
            tiles.iter().map(|t| t.source.clone()).collect(),
        );
        frame.map_id = MAP.into();
        frame.grid_origin = IVec2::ZERO;
        frame.source_map_size_core_tiles = Some(UVec2::new(8, 8));
        frame.boundary_warps = room
            .events
            .warps
            .iter()
            .map(|p| IVec2::new(p.x.into(), p.y.into()))
            .collect();
        let mut shell = TerrainMeshData {
            footing_heights: vec![2.75; w * h],
            ..Default::default()
        };
        perimeter(&mut shell, &frame, &refs, &g);
        assert_eq!(shell.camera_wall_textured_ranges.len(), 22);
        for (range, plane) in &shell.camera_wall_textured_ranges {
            let mut faces = 0;
            for triangle in shell.textured.indices.chunks_exact(3).filter(|t| {
                t.iter().all(|&i| range.contains(&(i as usize)))
                    && t.iter().any(|&i| {
                        shell.textured.uvs[i as usize] != shell.textured.uvs[t[0] as usize]
                    })
            }) {
                faces += 1;
                for &i in triangle {
                    let point = shell.textured.positions[i as usize];
                    let distance = plane[0] * point[0] + plane[1] * point[2] - plane[2];
                    assert!(
                        (distance - 0.02).abs() < 0.001,
                        "source trim must clear its solid backing"
                    );
                }
            }
            assert!(faces > 0, "each enclosure segment keeps native ornament");
        }
        assert_eq!(shell.footing_heights, vec![2.75; w * h]);
        for x in [56., 72.] {
            for y in [1., 12., 23.] {
                assert!(
                    clear(&shell.textured, Vec3::new(x, y, 129.5)),
                    "both source south exits stay open"
                );
            }
        }
        for x in [24., 40., 88., 104.] {
            assert!(
                !clear(&shell.textured, Vec3::new(x, 24., 128.)),
                "south course stays connected outside the exits"
            );
        }
        for bad in 0..w * 4 {
            let mut changed = tiles.clone();
            changed[bad].source.tile_index ^= 0x80;
            let mut rejected = TerrainMeshData::default();
            perimeter(
                &mut rejected,
                &frame,
                &changed.iter().collect::<Vec<_>>(),
                &g,
            );
            assert!(
                rejected.textured.indices.is_empty(),
                "changed rear cell{bad} cannot infer an enclosure"
            );
        }
        // A padded streaming grid must produce the same world-space shell.
        let margin = 7;
        let (bw, bh) = (w + margin * 2, h + margin * 2);
        let mut shifted = vec![tiles[w * h - 1].clone(); bw * bh];
        for y in 0..h {
            for x in 0..w {
                shifted[(y + margin) * bw + x + margin] = tiles[y * w + x].clone();
            }
        }
        let big = GridGeometry {
            width: bw,
            height: bh,
            origin_x: -(margin as f32) * 8.,
            origin_z: -(margin as f32) * 8.,
            ..g
        };
        frame.grid_origin = IVec2::splat(-(margin as i32));
        let mut shifted_shell = TerrainMeshData::default();
        perimeter(
            &mut shifted_shell,
            &frame,
            &shifted.iter().collect::<Vec<_>>(),
            &big,
        );
        assert_eq!(shell.textured.positions, shifted_shell.textured.positions);
        assert_eq!(shell.textured.normals, shifted_shell.textured.normals);
        assert_eq!(shell.textured.indices, shifted_shell.textured.indices);
        frame.map_id = "UnknownTowerRoom".into();
        let mut rejected = TerrainMeshData::default();
        perimeter(
            &mut rejected,
            &frame,
            &shifted.iter().collect::<Vec<_>>(),
            &big,
        );
        assert!(rejected.textured.indices.is_empty());
    }
}
