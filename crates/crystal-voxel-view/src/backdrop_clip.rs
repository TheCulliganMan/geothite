//! The active apron fills unknown scenery only. A neighbor may have a lower
//! authored floor (the ports do), so lowering the apron is not a valid join.
use crate::{TerrainRevisionCache, mesh::SurfaceMeshData, retained_terrain_transform};
use bevy::{mesh::VertexAttributeValues, prelude::*};
use crystal_render_api::VisualWorldNeighborhood;

pub(super) struct Backdrop {
    pub entity: Entity,
    handle: Handle<Mesh>,
    quads: Vec<Quad>,
    exclusions: Vec<Rect>,
}

struct Quad {
    rect: Rect,
    height: f32,
    uv: Rect,
    color: [f32; 4],
}

impl Backdrop {
    pub(super) fn new(mesh: &Mesh, handle: Handle<Mesh>, entity: Entity) -> Self {
        let Some(VertexAttributeValues::Float32x3(points)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("background positions");
        };
        let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0)
        else {
            panic!("background UVs");
        };
        let Some(VertexAttributeValues::Float32x4(colors)) = mesh.attribute(Mesh::ATTRIBUTE_COLOR)
        else {
            panic!("background colors");
        };
        let quads = points
            .chunks_exact(4)
            .enumerate()
            .map(|(i, p)| Quad {
                rect: Rect::from_corners(Vec2::new(p[0][0], p[0][2]), Vec2::new(p[2][0], p[2][2])),
                height: p[0][1],
                uv: Rect::from_corners(
                    Vec2::from_array(uvs[i * 4]),
                    Vec2::from_array(uvs[i * 4 + 2]),
                ),
                color: colors[i * 4],
            })
            .collect();
        Self {
            entity,
            handle,
            quads,
            exclusions: Vec::new(),
        }
    }
}

fn subtract(source: Rect, hole: Rect, result: &mut Vec<Rect>) {
    let low = source.min.max(hole.min);
    let high = source.max.min(hole.max);
    if !high.cmpgt(low).all() {
        result.push(source);
        return;
    }
    for (min, max) in [
        (source.min, Vec2::new(low.x, source.max.y)),
        (Vec2::new(high.x, source.min.y), source.max),
        (Vec2::new(low.x, source.min.y), Vec2::new(high.x, low.y)),
        (Vec2::new(low.x, high.y), Vec2::new(high.x, source.max.y)),
    ] {
        if max.cmpgt(min).all() {
            result.push(Rect { min, max });
        }
    }
}

fn clipped(quads: &[Quad], exclusions: &[Rect]) -> Mesh {
    let mut out = SurfaceMeshData::default();
    for quad in quads {
        let mut pieces = vec![quad.rect];
        for &hole in exclusions {
            let mut next = Vec::new();
            for piece in pieces {
                subtract(piece, hole, &mut next);
            }
            pieces = next;
        }
        for rect in pieces {
            let first = out.positions.len() as u32;
            for p in [
                rect.min,
                Vec2::new(rect.min.x, rect.max.y),
                rect.max,
                Vec2::new(rect.max.x, rect.min.y),
            ] {
                let t = (p - quad.rect.min) / quad.rect.size();
                out.positions.push([p.x, quad.height, p.y]);
                out.normals.push([0., 1., 0.]);
                out.uvs.push((quad.uv.min + t * quad.uv.size()).to_array());
                out.colors.push(quad.color);
            }
            out.indices
                .extend([first, first + 1, first + 2, first, first + 2, first + 3]);
        }
    }
    out.into_mesh()
}

pub(super) fn sync(
    frame: &crystal_render_api::VisualWorldFrame,
    neighborhood: &VisualWorldNeighborhood,
    cache: &mut TerrainRevisionCache,
    meshes: &mut Assets<Mesh>,
) {
    let Some(transform) = retained_terrain_transform(frame, cache) else {
        return;
    };
    let offset = transform.translation.xz();
    let Some(backdrop) = &mut cache.backdrop else {
        return;
    };
    let exclusions: Vec<_> = neighborhood
        .maps
        .iter()
        .map(|map| {
            let center = Vec2::new(map.center.x, -map.center.y) - offset;
            Rect::from_center_size(center, map.frame.grid_size.as_vec2() * map.frame.tile_size)
        })
        .collect();
    if exclusions.len() == backdrop.exclusions.len()
        && exclusions
            .iter()
            .zip(&backdrop.exclusions)
            .all(|(a, b)| a.min.abs_diff_eq(b.min, 0.001) && a.max.abs_diff_eq(b.max, 0.001))
    {
        return;
    }
    let mesh = clipped(&backdrop.quads, &exclusions);
    meshes
        .insert(backdrop.handle.id(), mesh)
        .expect("retained apron mesh handle");
    backdrop.exclusions = exclusions;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn neighbor_holes_remove_coverage_and_keep_source_texels_registered() {
        let quad = Quad {
            rect: Rect::from_corners(Vec2::splat(-8.), Vec2::splat(8.)),
            height: -0.04,
            uv: Rect::from_corners(Vec2::splat(-1.), Vec2::splat(1.)),
            color: [1.; 4],
        };
        let hole = Rect::from_corners(Vec2::new(-4., -8.), Vec2::new(4., 3.));
        let mesh = clipped(&[quad], &[hole]);
        let VertexAttributeValues::Float32x3(points) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap()
        else {
            panic!();
        };
        let VertexAttributeValues::Float32x2(uvs) = mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap()
        else {
            panic!();
        };
        let mut area = 0.;
        for p in points.chunks_exact(4) {
            let min = Vec2::new(p[0][0], p[0][2]);
            let max = Vec2::new(p[2][0], p[2][2]);
            let overlap = max.min(hole.max) - min.max(hole.min);
            assert!(
                !overlap.cmpgt(Vec2::ZERO).all(),
                "no apron may cover the lower neighboring floor"
            );
            let extent = max - min;
            area += extent.x * extent.y;
        }
        assert_eq!(area, 16. * 16. - 8. * 11.);
        for (p, uv) in points.iter().zip(uvs) {
            assert_eq!(p[1], -0.04);
            assert_eq!(uv[0], p[0] / 8.);
            assert_eq!(uv[1], p[2] / 8.);
        }
    }
}
