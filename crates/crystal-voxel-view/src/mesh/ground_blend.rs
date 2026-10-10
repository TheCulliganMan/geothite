//! Blend only verified, level path/lawn seams. Source occupancy stays exact.
use super::*;
#[derive(Default, Clone)]
pub(super) struct GroundBlend {
    edges: Vec<Edge>,
    cells: Vec<Vec<usize>>,
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
    pub(super) fn new(materials: &[Option<GroundMaterial>], g: &GridGeometry) -> Self {
        let mut result = Self {
            edges: Vec::new(),
            cells: vec![Vec::new(); materials.len()],
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
        let candidates = self.cells.get(cell)?;
        let width = g.tile_width.min(g.tile_height) * 0.36;
        let mut best = None;
        let mut nearest = f32::INFINITY;
        for &index in candidates {
            let edge = &self.edges[index];
            let other = 2 - edge.axis;
            let distance =
                (p[edge.axis] - edge.line).hypot(p[other] - p[other].clamp(edge.from, edge.to));
            if distance < width && distance < nearest {
                nearest = distance;
                best = Some(edge);
            }
        }
        let edge = best?;
        let map_p = [
            p[0] - g.origin_x + origin[0] as f32 * g.tile_width,
            p[1],
            p[2] - g.origin_z + origin[1] as f32 * g.tile_height,
        ];
        // A continuous world-space edge wiggle avoids a repeated tile stencil.
        let along = map_p[2 - edge.axis] / g.tile_width;
        let wobble = (meadow_noise(along * 2.3, 71.0) - 0.5) * g.tile_width * 0.14;
        let signed = (p[edge.axis] - edge.line) * if edge.path_positive { 1. } else { -1. };
        let t = ((signed + wobble) / width * 0.5 + 0.5).clamp(0., 1.);
        let t = t * t * (3. - 2. * t);
        let lawn = ground_color(GroundMaterial::Lawn, map_p, g);
        let path = ground_color(GroundMaterial::Path, map_p, g);
        Some(std::array::from_fn(|i| lawn[i] + (path[i] - lawn[i]) * t))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        assert!(
            GroundBlend::new(
                &[Some(GroundMaterial::Water), Some(GroundMaterial::Path)],
                &g
            )
            .edges
            .is_empty()
        );
        assert!(
            GroundBlend::new(&[None, Some(GroundMaterial::Path)], &g)
                .edges
                .is_empty()
        );
        let shifted_g = GridGeometry { origin_x: -8., ..g };
        let shifted = GroundBlend::new(
            &[Some(GroundMaterial::Lawn), Some(GroundMaterial::Path)],
            &shifted_g,
        );
        let scrolled = shifted.color([-0.001, 0., 4.], &shifted_g, [0, 0]).unwrap();
        assert!(
            left.iter()
                .zip(scrolled)
                .all(|(a, b)| (*a - b).abs() < 0.000001)
        );
    }
}
