//! Source-scoped damp soil on the land side of ordinary Johto water edges.
use super::*;
use bevy::prelude::Vec2;

#[derive(Clone, Default)]
pub(super) struct ShoreBlend {
    edges: Vec<([f32; 2], [f32; 2])>,
    candidates: Vec<Vec<usize>>,
    lawn: Vec<bool>,
}
impl ShoreBlend {
    pub(super) fn new(map: &str, cells: &[&VisualTile], g: &GridGeometry) -> Self {
        if !ordinary_lawn_map(map) {
            return Self::default();
        }
        let lawn: Vec<_> = cells
            .iter()
            .map(|t| {
                matches!(t.source.tileset_id.as_ref(), "johto" | "johto_modern")
                    && ground_material(&t.source) == Some(GroundMaterial::Lawn)
            })
            .collect();
        let mut result = Self {
            edges: Vec::new(),
            candidates: vec![Vec::new(); cells.len()],
            lawn,
        };
        for row in 0..g.height {
            for col in 0..g.width {
                let i = row * g.width + col;
                if !result.lawn[i] {
                    continue;
                }
                let (w, e, n, s) = g.bounds(col, row);
                for direction in Direction::ALL {
                    let (dx, dz) = direction.offset();
                    let x = col as isize + dx;
                    let z = row as isize + dz;
                    if x < 0 || z < 0 || x >= g.width as isize || z >= g.height as isize {
                        continue;
                    }
                    let other = &cells[z as usize * g.width + x as usize].source;
                    if other.tileset_id != cells[i].source.tileset_id
                        || !matches!(shape_for_source(other), CellShape::Water)
                    {
                        continue;
                    }
                    let edge = match direction {
                        Direction::North => ([w, n], [e, n]),
                        Direction::South => ([w, s], [e, s]),
                        Direction::West => ([w, n], [w, s]),
                        Direction::East => ([e, n], [e, s]),
                    };
                    let index = result.edges.len();
                    result.edges.push(edge);
                    for y in row.saturating_sub(1)..=(row + 1).min(g.height - 1) {
                        for x in col.saturating_sub(1)..=(col + 1).min(g.width - 1) {
                            result.candidates[y * g.width + x].push(index);
                        }
                    }
                }
            }
        }
        result
    }
    pub(super) fn near(&self, p: [f32; 3], g: &GridGeometry) -> bool {
        cell_at(g, p[0], p[2]).is_some_and(|i| {
            self.lawn.get(i).copied().unwrap_or(false)
                && self.candidates.get(i).is_some_and(|v| !v.is_empty())
        })
    }
    pub(super) fn color(
        &self,
        p: [f32; 3],
        g: &GridGeometry,
        origin: [i32; 2],
        base: [f32; 4],
    ) -> [f32; 4] {
        if p[1].abs() > g.tile_height * 0.0001 {
            return base;
        }
        let Some(i) = cell_at(g, p[0], p[2]) else {
            return base;
        };
        let Some(candidates) = self.candidates.get(i) else {
            // Excluded maps intentionally have no shoreline index. Their
            // ground finish must remain a no-op instead of indexing it.
            return base;
        };
        let owned = self.lawn.get(i).copied().unwrap_or(false);
        let x = (p[0] - g.origin_x) / g.tile_width + origin[0] as f32;
        let z = (p[2] - g.origin_z) / g.tile_height + origin[1] as f32;
        let width = g.tile_width.min(g.tile_height)
            * (0.55 + meadow_noise(x * 2.1 + 37., z * 2.1 - 17.) * 0.22);
        let mut amount = 0.0_f32;
        for &index in candidates {
            let (a, b) = self.edges[index];
            let a = Vec2::from_array(a);
            let b = Vec2::from_array(b);
            let q = Vec2::new(p[0], p[2]);
            let d = b - a;
            let t = ((q - a).dot(d) / d.length_squared()).clamp(0., 1.);
            let distance = (q - a - d * t).length();
            // A shared lawn vertex can lie exactly on the water cell's
            // half-open grid boundary. Retain its bank color on that line,
            // without painting any point inside the water or unknown cells.
            if !owned && distance > g.tile_width.min(g.tile_height) * 0.0001 {
                continue;
            }
            let weight = (1. - distance / width).clamp(0., 1.);
            amount = amount.max(weight * weight * (3. - 2. * weight) * 0.62);
        }
        let soil = ledge_finish::soil_color(x, z, 0.45);
        [
            base[0] + (soil[0] - base[0]) * amount,
            base[1] + (soil[1] - base[1]) * amount,
            base[2] + (soil[2] - base[2]) * amount,
            base[3],
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::tests::{frame, source_with_tile};
    use super::*;
    fn fixture() -> VisualWorldFrame {
        frame(
            3,
            2,
            vec![
                source_with_tile(0x01, 0, 0, 0x05),
                source_with_tile(0x35, 0, 0, 0x14),
                source_with_tile(0x35, 0, 0, 0x14),
                source_with_tile(0x01, 0, 1, 0x05),
                source_with_tile(0x35, 0, 1, 0x14),
                source_with_tile(0x35, 0, 1, 0x14),
            ],
        )
    }
    fn grid() -> GridGeometry {
        GridGeometry {
            width: 3,
            height: 2,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        }
    }
    #[test]
    fn excluded_maps_leave_ground_colors_unchanged_at_in_bounds_cells() {
        let f = fixture();
        let g = grid();
        let cells: Vec<_> = f.tiles.iter().collect();
        let base = [0.31, 0.42, 0.12, 1.];
        for map in ["IlexForest", "UnionCave1F", "Route43Gate"] {
            let blend = ShoreBlend::new(map, &cells, &g);
            for row in 0..g.height {
                for column in 0..g.width {
                    let p = [(column as f32 + 0.5) * 8., 0., (row as f32 + 0.5) * 8.];
                    assert!(!blend.near(p, &g));
                    assert_eq!(blend.color(p, &g, [0, 0], base), base, "{map}");
                }
            }
        }
    }
    #[test]
    fn wet_edge_is_bounded_continuous_and_map_anchored() {
        let f = fixture();
        let g = grid();
        let cells: Vec<_> = f.tiles.iter().collect();
        let blend = ShoreBlend::new("LakeOfRage", &cells, &g);
        let base = [0.31, 0.42, 0.12, 1.];
        assert_eq!(blend.edges.len(), 2);
        assert_eq!(blend.color([0., 0., 4.], &g, [0, 0], base), base);
        assert_eq!(blend.color([8.5, 0., 4.], &g, [0, 0], base), base);
        assert_eq!(blend.color([7., 2., 4.], &g, [0, 0], base), base);
        let edge = blend.color([8., 0., 4.], &g, [0, 0], base);
        assert_ne!(edge, base);
        let nearby = blend.color([7.999, 0., 4.], &g, [0, 0], base);
        assert!((0..4).all(|i| (edge[i] - nearby[i]).abs() < 0.001));
        let a = blend.color([7., 0., 7.999], &g, [0, 0], base);
        let b = blend.color([7., 0., 8.001], &g, [0, 0], base);
        assert!((0..4).all(|i| (a[i] - b[i]).abs() < 0.001));
        let moved = GridGeometry {
            origin_x: -80.,
            origin_z: 48.,
            ..g
        };
        let shifted = ShoreBlend::new("LakeOfRage", &cells, &moved);
        assert_eq!(
            blend.color([7., 0., 4.], &g, [13, 17], base),
            shifted.color([-73., 0., 52.], &moved, [13, 17], base)
        );
        let padded = frame(
            4,
            2,
            vec![
                source_with_tile(0x01, 0, 0, 0x05),
                f.tiles[0].source.clone(),
                f.tiles[1].source.clone(),
                f.tiles[2].source.clone(),
                source_with_tile(0x01, 0, 1, 0x05),
                f.tiles[3].source.clone(),
                f.tiles[4].source.clone(),
                f.tiles[5].source.clone(),
            ],
        );
        let padded_grid = GridGeometry {
            width: 4,
            origin_x: -8.,
            ..g
        };
        let padded_cells: Vec<_> = padded.tiles.iter().collect();
        let shifted = ShoreBlend::new("LakeOfRage", &padded_cells, &padded_grid);
        assert_eq!(
            blend.color([7., 0., 4.], &g, [13, 17], base),
            shifted.color([7., 0., 4.], &padded_grid, [12, 17], base)
        );
    }
    #[test]
    fn finished_damp_bank_retains_native_floor_corners_heights_and_claims() {
        let mut f = fixture();
        f.map_id = std::sync::Arc::from("LakeOfRage");
        let g = GridGeometry {
            origin_x: -12.,
            origin_z: -8.,
            ..grid()
        };
        let cells: Vec<_> = f.tiles.iter().collect();
        let mut mesh =
            build_terrain_mesh_with_samples(&f, &TerrainImageSamples::default()).unwrap();
        let corners: Vec<_> = mesh
            .textured
            .positions
            .iter()
            .zip(&mesh.textured.normals)
            .filter(|(p, n)| p[1] == 0. && n[1] > 0.999)
            .map(|(p, _)| *p)
            .collect();
        assert!(!corners.is_empty());
        let heights = mesh.footing_heights.clone();
        let claims = mesh.authored_cells.clone();
        polish_world_surfaces(&mut mesh, "LakeOfRage", &cells, &g, [0, 0]);
        assert_eq!(mesh.footing_heights, heights);
        assert_eq!(mesh.authored_cells, claims);
        for corner in corners {
            assert!(
                mesh.solid
                    .positions
                    .iter()
                    .zip(&mesh.solid.normals)
                    .chain(mesh.textured.positions.iter().zip(&mesh.textured.normals))
                    .any(|(p, n)| *p == corner && n[1] > 0.999)
            );
        }
    }
    #[test]
    fn missing_water_unknown_atlas_and_indoor_views_keep_original_ground() {
        let f = fixture();
        let g = grid();
        let cells: Vec<_> = f.tiles.iter().collect();
        assert!(ShoreBlend::new("Route43Gate", &cells, &g).edges.is_empty());
        let mut f = f;
        for t in &mut f.tiles {
            t.source = source_with_tile(0x01, 0, 0, 0x05);
        }
        let cells: Vec<_> = f.tiles.iter().collect();
        assert!(ShoreBlend::new("LakeOfRage", &cells, &g).edges.is_empty());
        let mut f = fixture();
        for t in &mut f.tiles {
            t.source.tileset_id = std::sync::Arc::from("unknown");
        }
        let cells: Vec<_> = f.tiles.iter().collect();
        assert!(ShoreBlend::new("LakeOfRage", &cells, &g).edges.is_empty());
    }
}
