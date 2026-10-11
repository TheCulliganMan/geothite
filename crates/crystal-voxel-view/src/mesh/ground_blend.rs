//! Blend only verified, matching path/lawn edges. Source occupancy stays exact.
use super::*;
#[derive(Default, Clone)]
pub(super) struct GroundBlend {
    edges: Vec<Edge>,
    cells: Vec<Vec<usize>>,
    materials: Vec<Option<GroundMaterial>>,
}
#[derive(Clone)]
struct Edge {
    axis: usize,
    line: f32,
    from: f32,
    to: f32,
    path_positive: bool,
}
impl GroundBlend {
    #[cfg(test)]
    pub(super) fn new(materials: &[Option<GroundMaterial>], g: &GridGeometry) -> Self {
        Self::build(materials, None, g)
    }
    #[cfg(test)]
    pub(super) fn at_heights(
        materials: &[Option<GroundMaterial>],
        heights: &[f32],
        g: &GridGeometry,
    ) -> Self {
        let corners: Vec<_> = heights.iter().map(|&h| [h; 4]).collect();
        Self::at_corners(materials, &corners, g)
    }
    pub(super) fn at_corners(
        materials: &[Option<GroundMaterial>],
        heights: &[[f32; 4]],
        g: &GridGeometry,
    ) -> Self {
        Self::build(materials, Some(heights), g)
    }
    fn build(
        materials: &[Option<GroundMaterial>],
        heights: Option<&[[f32; 4]]>,
        g: &GridGeometry,
    ) -> Self {
        let mut result = Self {
            edges: Vec::new(),
            cells: vec![Vec::new(); materials.len()],
            materials: materials.to_vec(),
        };
        for row in 0..g.height {
            for col in 0..g.width {
                let i = row * g.width + col;
                for (axis, other) in [
                    (0, (col + 1 < g.width).then_some(i + 1)),
                    (2, (row + 1 < g.height).then_some(i + g.width)),
                ] {
                    let Some(other) = other else {
                        continue;
                    };
                    let pairs = if axis == 0 {
                        [(3, 0), (2, 1)]
                    } else {
                        [(1, 0), (2, 3)]
                    };
                    if heights.is_some_and(|h| {
                        pairs
                            .iter()
                            .any(|&(a, b)| (h[i][a] - h[other][b]).abs() > 0.001)
                    }) {
                        continue;
                    }
                    let (Some(a), Some(b)) = (materials[i], materials[other]) else {
                        continue;
                    };
                    if !matches!(
                        (a, b),
                        (GroundMaterial::Path, GroundMaterial::Lawn)
                            | (GroundMaterial::Lawn, GroundMaterial::Path)
                    ) {
                        continue;
                    }
                    let (w, e, n, s) = g.bounds(col, row);
                    let edge = if axis == 0 {
                        Edge {
                            axis,
                            line: e,
                            from: n,
                            to: s,
                            path_positive: b == GroundMaterial::Path,
                        }
                    } else {
                        Edge {
                            axis,
                            line: s,
                            from: w,
                            to: e,
                            path_positive: b == GroundMaterial::Path,
                        }
                    };
                    let index = result.edges.len();
                    result.edges.push(edge);
                    // Include diagonal corner candidates so adjacent faces sample
                    // the same transition instead of exposing a tile-shaped seam.
                    for z in row.saturating_sub(1)..=(row + 1).min(g.height - 1) {
                        for x in col.saturating_sub(1)..=(col + 1).min(g.width - 1) {
                            result.cells[z * g.width + x].push(index);
                        }
                    }
                }
            }
        }
        result
    }
    pub(super) fn near(&self, p: [f32; 3], g: &GridGeometry) -> bool {
        cell_at(g, p[0], p[2])
            .and_then(|i| self.cells.get(i))
            .is_some_and(|v| !v.is_empty())
    }
    pub(super) fn color(
        &self,
        p: [f32; 3],
        g: &GridGeometry,
        origin: [i32; 2],
    ) -> Option<[f32; 4]> {
        let cell = cell_at(g, p[0], p[2])?;
        let material = self.materials.get(cell).copied().flatten()?;
        if !matches!(material, GroundMaterial::Lawn | GroundMaterial::Path) {
            return None;
        }
        let candidates = self.cells.get(cell)?;
        let width = g.tile_width.min(g.tile_height) * 0.65;
        let map_p = [
            p[0] - g.origin_x + origin[0] as f32 * g.tile_width,
            p[1],
            p[2] - g.origin_z + origin[1] as f32 * g.tile_height,
        ];
        let mut nearest = f32::INFINITY;
        let mut path_weight = 0.;
        let mut total_weight = 0.;
        for &index in candidates {
            let edge = &self.edges[index];
            let other = 2 - edge.axis;
            let distance =
                (p[edge.axis] - edge.line).hypot(p[other] - p[other].clamp(edge.from, edge.to));
            if distance >= width {
                continue;
            }
            nearest = nearest.min(distance);
            // A continuous world-space edge wiggle avoids a repeated stencil.
            let along = map_p[other] / g.tile_width;
            let across = (edge.line - if edge.axis == 0 { g.origin_x } else { g.origin_z })
                / g.tile_width + origin[edge.axis / 2] as f32;
            let wobble = (meadow_noise(along * 1.7, across * 0.37 + 71.0)
                + meadow_noise(along * 4.3, across * 0.61 - 19.0) * 0.35)
                * g.tile_width * 0.45;
            let signed = (p[edge.axis] - edge.line) * if edge.path_positive { 1. } else { -1. };
            let t = ((signed + wobble) / width * 0.5 + 0.5).clamp(0., 1.);
            let t = t * t * (3. - 2. * t);
            // Nearest-edge selection jumped between different wobble fields
            // at corners. Combine them with weights that vanish at their ends.
            let weight = (1. - distance / width).powi(2);
            path_weight += t * weight;
            total_weight += weight;
        }
        if total_weight <= 1e-12 {
            return None;
        }
        let t = path_weight / total_weight;
        let lawn = ground_color(GroundMaterial::Lawn, map_p, g);
        let path = ground_color(GroundMaterial::Path, map_p, g);
        let base = ground_color(material, map_p, g);
        // Wobble can retain mixed color at the geometric cutoff. Feather the
        // outer half back to the owned material before leaving the blend band.
        let fade = (nearest / width * 2. - 1.).clamp(0., 1.);
        let amount = 1. - fade * fade * (3. - 2. * fade);
        Some(std::array::from_fn(|i| {
            let mixed = lawn[i] + (path[i] - lawn[i]) * t;
            base[i] + (mixed - base[i]) * amount
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wobbled_band_rejoins_unblended_ground_without_a_color_step() {
        let g = GridGeometry { width: 2, height: 1, tile_width: 8., tile_height: 8.,
            origin_x: 0., origin_z: 0. };
        let materials = [Some(GroundMaterial::Lawn), Some(GroundMaterial::Path)];
        let blend = GroundBlend::new(&materials, &g);
        let width = g.tile_width * 0.65;
        for origin in [[0,0],[-31,-7],[17,43],[64,-32]] {
            for (sign, material) in [(-1., GroundMaterial::Lawn),(1., GroundMaterial::Path)] {
                let at = |distance| [8. + sign * distance, 0., 4.];
                let sample = |p: [f32;3]| blend.color(p, &g, origin).unwrap_or_else(||
                    ground_color(material, [p[0] + origin[0] as f32 * 8., p[1],
                        p[2] + origin[1] as f32 * 8.], &g));
                let inside = sample(at(width - 0.001));
                let outside = sample(at(width + 0.001));
                assert!(inside.iter().zip(outside).all(|(a,b)| (a-b).abs() < 0.0001),
                    "the outer blend boundary must be continuous at source phase {origin:?}");
            }
        }
    }
    #[test]
    fn adjoining_edges_blend_continuously_across_the_corner_bisector() {
        let g = GridGeometry { width: 3, height: 3, tile_width: 8., tile_height: 8.,
            origin_x: 0., origin_z: 0. };
        let mut materials = [Some(GroundMaterial::Lawn);9];
        for i in [0,1,3] { materials[i] = Some(GroundMaterial::Path); }
        let blend = GroundBlend::new(&materials, &g);
        for origin in [[0,0],[-31,-7],[17,43],[64,-32]] {
            for distance in [0.5,1.,2.,3.] {
                let a = blend.color([8.+distance-0.001,0.,8.+distance+0.001], &g, origin).unwrap();
                let b = blend.color([8.+distance+0.001,0.,8.+distance-0.001], &g, origin).unwrap();
                assert!(a.iter().zip(b).all(|(a,b)| (a-b).abs() < 0.001),
                    "overlapping edges must not switch pigment at their bisector");
            }
        }
    }
    #[test]
    fn sloping_ledge_toe_blends_but_its_front_drop_remains_distinct() {
        let g = GridGeometry {
            width: 1,
            height: 3,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let materials = [
            Some(GroundMaterial::Path),
            Some(GroundMaterial::Lawn),
            Some(GroundMaterial::Path),
        ];
        let blend = GroundBlend::at_corners(&materials, &[[0.; 4], [0., 6., 6., 0.], [0.; 4]], &g);
        assert_eq!(blend.edges.len(), 1);
        assert_eq!(blend.edges[0].line, 8.);
        let a = blend.color([4., 0., 7.999], &g, [0, 0]).unwrap();
        let b = blend.color([4., 0., 8.001], &g, [0, 0]).unwrap();
        assert!(a.iter().zip(b).all(|(a, b)| (a - b).abs() < 0.001));
        assert!(blend.color([4., 6., 16.], &g, [0, 0]).is_none());
        let side = GridGeometry {
            width: 2,
            height: 1,
            ..g
        };
        assert!(
            GroundBlend::at_corners(&materials[..2], &[[0.; 4], [0., 6., 6., 0.]], &side)
                .edges
                .is_empty(),
            "one matching corner cannot blend a mismatched side edge"
        );
    }

    #[test]
    fn cap_transitions_blend_within_a_tier_and_stop_at_a_jump_step() {
        let g = GridGeometry {
            width: 2,
            height: 1,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let materials = [Some(GroundMaterial::Lawn), Some(GroundMaterial::Path)];
        assert_eq!(
            GroundBlend::at_heights(&materials, &[3., 3.], &g)
                .edges
                .len(),
            1
        );
        assert!(GroundBlend::at_heights(&materials, &[0., 3.], &g)
            .edges
            .is_empty());
    }

    #[test]
    fn only_verified_lawn_path_edges_blend_and_the_band_is_continuous() {
        let g = GridGeometry {
            width: 2,
            height: 1,
            tile_width: 8.,
            tile_height: 8.,
            origin_x: 0.,
            origin_z: 0.,
        };
        let blend = GroundBlend::new(
            &[Some(GroundMaterial::Lawn), Some(GroundMaterial::Path)],
            &g,
        );
        assert_eq!(blend.edges.len(), 1);
        let left = blend.color([7.999, 0., 4.], &g, [0, 0]).unwrap();
        let right = blend.color([8.001, 0., 4.], &g, [0, 0]).unwrap();
        assert!(left.iter().zip(right).all(|(a, b)| (*a - b).abs() < 0.001));
        assert!(blend.color([1., 0., 4.], &g, [0, 0]).is_none());
        assert!(GroundBlend::new(
            &[Some(GroundMaterial::Water), Some(GroundMaterial::Path)],
            &g
        )
        .edges
        .is_empty());
        assert!(GroundBlend::new(&[None, Some(GroundMaterial::Path)], &g)
            .edges
            .is_empty());
        let shifted_g = GridGeometry { origin_x: -8., ..g };
        let shifted = GroundBlend::new(
            &[Some(GroundMaterial::Lawn), Some(GroundMaterial::Path)],
            &shifted_g,
        );
        let scrolled = shifted.color([-0.001, 0., 4.], &shifted_g, [0, 0]).unwrap();
        assert!(left
            .iter()
            .zip(scrolled)
            .all(|(a, b)| (*a - b).abs() < 0.000001));
    }
}
