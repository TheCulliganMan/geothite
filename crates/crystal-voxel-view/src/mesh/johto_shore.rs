//! Shared rocky ledges at the existing town shoreline and water drop.
//! The source footprint, terrain backing and swimming/walking datums remain.
use super::*;

const LABEL: &str = "outdoor/johto-shore";

fn eligible(map: &str) -> bool {
    map.ends_with("City")
        || map.ends_with("Town")
        || matches!(map, "LakeOfRage" | "SilverCaveOutside")
        || map
            .strip_prefix("Route")
            .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}

// A complete rocky $3d cap has outline, ochre body and lit mineral. Sampling
// that native role avoids promoting water blue or pale neighboring ground.
fn palette(
    cells: &[&VisualTile],
    shapes: &[CellShape],
    family: &str,
    images: &TerrainImageSamples,
) -> Option<[[f32; 3]; 2]> {
    for (i, tile) in cells.iter().enumerate() {
        if shapes[i] != CellShape::ShoreBand
            || tile.source.tileset_id.as_ref() != family
            || tile.source.tile_index != 0x3d
        {
            continue;
        }
        let rgba = tile_rgba(images, tile).ok()?;
        if rgba.len() != 256 || rgba.chunks_exact(4).any(|p| p[3] != 255) {
            continue;
        }
        let colors = rgba
            .chunks_exact(4)
            .map(|p| [p[0], p[1], p[2]])
            .collect::<std::collections::BTreeSet<_>>();
        if colors.len() != 3 {
            continue;
        }
        let mut colors = colors.into_iter().collect::<Vec<_>>();
        colors.sort_by_key(|c| {
            u32::from(c[0]) * 2126 + u32::from(c[1]) * 7152 + u32::from(c[2]) * 722
        });
        let linear = |c: [u8; 3]| {
            c.map(|v| {
                let s = f32::from(v) / 255.;
                if s <= 0.04045 {
                    s / 12.92
                } else {
                    ((s + 0.055) / 1.055).powf(2.4)
                }
            })
        };
        return Some([linear(colors[2]), linear(colors[1])]);
    }
    None
}

fn water_neighbor(i: usize, shapes: &[CellShape], g: &GridGeometry, dx: isize, dz: isize) -> bool {
    let x = (i % g.width) as isize + dx;
    let z = (i / g.width) as isize + dz;
    x >= 0 && z >= 0 && x < g.width as isize && z < g.height as isize
        && shapes[z as usize * g.width + x as usize] == CellShape::Water
}

fn bank(i: usize, cells: &[&VisualTile], shapes: &[CellShape], g: &GridGeometry) -> bool {
    shapes[i] == CellShape::ShoreBand
        || (shapes[i] == CellShape::Flat
            && matches!((cells[i].source.tileset_id.as_ref(), cells[i].source.tile_index),
                ("johto" | "johto_modern", 0x05 | 0x06))
            && Direction::ALL.into_iter().any(|d| {
                let (dx, dz) = d.offset();
                water_neighbor(i, shapes, g, dx, dz)
            }))
}

pub(super) fn finish(
    mesh: &mut TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    shapes: &[CellShape],
    g: &GridGeometry,
    origin: [i32; 2],
    images: &TerrainImageSamples,
) {
    if !eligible(map)
        || cells.len() != g.width * g.height
        || shapes.len() != cells.len()
        || mesh.authored_cells.len() != cells.len()
    {
        return;
    }
    let old = &mesh.textured;
    if old.positions.len() % 4 != 0
        || old.indices.len() != old.positions.len() / 4 * 6
        || old.indices.chunks_exact(6).enumerate().any(|(q, i)| {
            let b = (q * 4) as u32;
            i != [b, b + 1, b + 2, b, b + 2, b + 3]
        })
    {
        return;
    }
    let mut faces = vec![Vec::new(); cells.len()];
    let mut bad = vec![false; cells.len()];
    for (q, uv) in old.uvs.chunks_exact(4).enumerate() {
        let Some(i) = new_bark::sampled_cell(uv, g) else {
            continue;
        };
        if !bank(i, cells, shapes, g)
            || !matches!(
                cells[i].source.tileset_id.as_ref(),
                "johto" | "johto_modern"
            )
        {
            continue;
        }
        let p = &old.positions[q * 4..q * 4 + 4];
        let n = old.normals[q * 4];
        if kanto_shore::native_face(p, n, uv, i, shapes, g)
            && old.normals[q * 4..q * 4 + 4].iter().all(|v| *v == n)
        {
            faces[i].push(q);
        } else {
            bad[i] = true;
        }
    }
    let palettes = ["johto", "johto_modern"].map(|family| palette(cells, shapes, family, images));
    let mut removed = std::collections::HashSet::new();
    for i in 0..cells.len() {
        if bad[i]
            || mesh.authored_cells[i].is_some()
            || !faces[i].iter().any(|&q| old.normals[q * 4] == [0., 1., 0.])
        {
            continue;
        }
        let Some(palette) =
            palettes[usize::from(cells[i].source.tileset_id.as_ref() == "johto_modern")]
        else {
            continue;
        };
        for &q in &faces[i] {
            new_bark::ledge_finish::append_natural_bank(
                &mut mesh.solid,
                old.positions[q * 4..q * 4 + 4].try_into().unwrap(),
                old.normals[q * 4],
                g,
                origin,
                palette,
            );
            removed.insert(q);
        }
        mesh.authored_cells[i] = Some(LABEL);
    }
    if removed.is_empty() {
        return;
    }
    let old = std::mem::take(&mut mesh.textured);
    for (_, anchor) in &mut mesh.cutaway_links {
        *anchor -= removed.iter().filter(|&&q| q < *anchor / 4).count() * 4;
    }
    let mut marked = vec![false; old.positions.len()];
    for r in &old.cutaway_ranges {
        marked[r.clone()].fill(true);
    }
    let mut remap = vec![u32::MAX; old.positions.len()];
    mesh.textured = select_surface_indices(
        &old,
        &marked,
        &mut remap,
        old.indices
            .chunks_exact(6)
            .enumerate()
            .filter(|(q, _)| !removed.contains(q))
            .flat_map(|(_, i)| i.iter().copied()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_lake_banks_share_the_cliff_face_on_all_four_edges_without_moving_water() {
        for direction in Direction::ALL {
            let g = GridGeometry { width: 3, height: 3, tile_width: 8., tile_height: 8.,
                origin_x: 0., origin_z: 0. };
            let (dx, dz) = direction.offset();
            let water = ((1 + dz) * 3 + 1 + dx) as usize;
            let mut handles = bevy::prelude::Assets::<bevy::prelude::Image>::default();
            let texture = handles.add(bevy::prelude::Image::default());
            let mut images = TerrainImageSamples::default();
            images.pixels.insert(texture.id(), TileImageSample::Rgba((0..64).flat_map(|p| {
                [[57, 57, 57, 255], [165, 123, 24, 255], [198, 148, 57, 255]][p % 3]
            }).collect()));
            let tiles = (0..9).map(|i| {
                let mut source = super::super::tests::source_with_tile(
                    if i == 0 { 0x54 } else { 0x01 }, 0, 0,
                    if i == 0 { 0x3d } else if i == water { 0x14 }
                    else if i == 4 { 0x06 } else { 0x99 });
                source.tileset_id = "johto".into();
                VisualTile { column: (i % 3) as u32, row: (i / 3) as u32,
                    source, texture: texture.clone(), animation_frames: None, priority: false }
            }).collect::<Vec<_>>();
            let cells = tiles.iter().collect::<Vec<_>>();
            let shapes = cells.iter().map(|t| shape_for_source(&t.source)).collect::<Vec<_>>();
            let mut mesh = TerrainMeshData::default();
            mesh.authored_cells = vec![None; 9];
            mesh.footing_heights = shapes.iter().map(|s| s.surface_height(8.)).collect();
            for (i, shape) in shapes.iter().enumerate() {
                append_top(&mut mesh.textured, g.bounds(i % 3, i / 3).into(),
                    shape.surface_height(8.), g.uv(i % 3, i / 3));
            }
            append_textured_shoreline(&mut mesh.textured, &g, 1, 1, direction,
                [8., 16., 8., 16.], crate::profile::WATER_HEIGHT, 0.);
            let footing = mesh.footing_heights.clone();
            let mut new_bark = mesh.clone();
            finish(&mut mesh, "EcruteakCity", &cells, &shapes, &g, [0, 0], &images);
            finish(&mut new_bark, "NewBarkTown", &cells, &shapes, &g, [0, 0], &images);
            assert_eq!(mesh.solid, new_bark.solid,
                "New Bark must use the same natural bank on every orientation");
            assert_eq!(mesh.authored_cells[4], Some(LABEL));
            assert_eq!(mesh.authored_cells[water], None);
            assert_eq!(mesh.footing_heights, footing);
            let drop = mesh.footing_heights[4] - mesh.footing_heights[water];
            assert!(drop > 0. && drop <= g.tile_height / 16.,
                "standing water must meet every bank orientation with a short drop: {drop}");
            assert_eq!(mesh.authored_cells.iter().filter(|v| v.is_some()).count(), 2,
                "unknown neighboring artwork must retain its own presentation");
            assert_eq!(mesh.textured.quad_count(), 7);
            assert!(mesh.textured.positions.iter().any(|p| p[1] == crate::profile::WATER_HEIGHT),
                "the liquid surface keeps its original datum");
            assert!(mesh.solid.positions.iter().all(|p|p[1]>=crate::profile::WATER_HEIGHT && p[1]<=0.),
                "shore ledges use the standard cliff without a separate pebble crown");
            let normal=Vec3::new(dx as f32,0.,dz as f32);
            let edge=Vec3::new(if dx>0 {16.}else{8.},0.,if dz>0 {16.}else{8.});
            assert!(mesh.solid.positions.iter().any(|&p| {
                let p=Vec3::from_array(p);let relief=(p-edge).dot(normal);
                p.y>crate::profile::WATER_HEIGHT && p.y<0. && relief>0.05 && relief<0.5
                    && if dx==0 {p.x>8.&&p.x<16.} else {p.z>8.&&p.z<16.}
            }),"all four directions keep physical cliff-wall relief below the existing cap");
        }
    }

    #[test]
    fn town_shore_finish_preserves_water_footing_and_rejects_foreign_claims() {
        let g = GridGeometry {
            width: 2,
            height: 2,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let mut handles = bevy::prelude::Assets::<bevy::prelude::Image>::default();
        let mut images = TerrainImageSamples::default();
        let mut tiles = Vec::new();
        let mut shapes = Vec::new();
        let mut mesh = TerrainMeshData::default();
        for i in 0..4 {
            let texture = handles.add(bevy::prelude::Image::default());
            images.pixels.insert(
                texture.id(),
                TileImageSample::Rgba(
                    (0..64)
                        .flat_map(|p| {
                            [[57, 57, 57, 255], [165, 123, 24, 255], [198, 148, 57, 255]][p % 3]
                        })
                        .collect(),
                ),
            );
            let mut source = super::super::tests::source_with_tile(
                0x54,
                (i % 2) as u8,
                (i / 2) as u8,
                if i == 0 { 0x3d } else { 0x14 },
            );
            source.tileset_id = "johto".into();
            let shape = shape_for_source(&source);
            shapes.push(shape);
            tiles.push(VisualTile {
                column: (i % 2) as u32,
                row: (i / 2) as u32,
                source,
                texture,
                animation_frames: None,
                priority: false,
            });
            mesh.authored_cells.push(None);
            mesh.footing_heights.push(shape.surface_height(8.));
            let (w, e, n, s) = g.bounds(i % 2, i / 2);
            append_top(
                &mut mesh.textured,
                [w, e, n, s],
                shape.surface_height(8.),
                g.uv(i % 2, i / 2),
            );
        }
        for d in [Direction::East, Direction::South] {
            append_textured_shoreline(&mut mesh.textured, &g, 0, 0, d, [0., 8., 0., 8.], crate::profile::WATER_HEIGHT, 0.);
        }
        let cells = tiles.iter().collect::<Vec<_>>();
        let original = mesh.clone();
        finish(
            &mut mesh,
            "OlivineCity",
            &cells,
            &shapes,
            &g,
            [0, 0],
            &images,
        );
        assert_eq!(mesh.footing_heights, original.footing_heights);
        assert_eq!(mesh.authored_cells, vec![Some(LABEL), None, None, None]);
        let caps = mesh.solid.normals.iter().enumerate()
            .filter(|(_, normal)| normal[1] > 0.99).collect::<Vec<_>>();
        assert!(!caps.is_empty());
        assert!(caps.iter().all(|(i, _)|
            mesh.solid.uvs[*i] == new_bark::ledge_finish::BANK_GROUND_UV),
            "horizontal banks must inherit dry ground rather than cliff paint");
        assert_eq!(
            mesh.textured.quad_count(),
            3,
            "all water faces retain their source atlas"
        );
        assert!(mesh.solid.indices.len() / 3 > 30 && mesh.solid.indices.len() / 3 < 500);
        assert!(
            mesh.solid
                .positions
                .iter()
                .all(|p| p[0] >= -3.6 && p[0] <= 11.6 && p[2] >= -3.6 && p[2] <= 11.6)
        );
        assert!(
            mesh.solid
                .uvs
                .iter()
                .all(|v| *v == crate::scenery_finish::SOURCE_PAINT_UV
                    || *v == new_bark::ledge_finish::BANK_GROUND_UV
                    || (v[0] > 87000. && v[0] < 90000. && v[1] < -10000.))
        );
        for map in ["PlayersHouse1F", "NationalPark"] {
            let mut unchanged = original.clone();
            finish(&mut unchanged, map, &cells, &shapes, &g, [0, 0], &images);
            assert_eq!(unchanged.textured, original.textured);
            assert!(unchanged.solid.positions.is_empty());
        }
        let mut claimed = original.clone();
        claimed.authored_cells[0] = Some("other");
        finish(
            &mut claimed,
            "OlivineCity",
            &cells,
            &shapes,
            &g,
            [0, 0],
            &images,
        );
        assert_eq!(claimed.textured, original.textured);
        assert!(claimed.solid.positions.is_empty());
        let mut missing = original.clone();
        finish(
            &mut missing,
            "OlivineCity",
            &cells,
            &shapes,
            &g,
            [0, 0],
            &TerrainImageSamples::default(),
        );
        assert_eq!(missing.textured, original.textured);
        assert!(missing.solid.positions.is_empty());
    }
}
