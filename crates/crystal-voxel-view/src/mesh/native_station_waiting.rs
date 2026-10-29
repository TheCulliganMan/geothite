//! Complete native timber-hall wall courses and their two separate notices.
use crate::interior_models::ModelKind;
use crate::mesh::{GridGeometry, SurfaceMeshData};
use crystal_render_api::{VisualTile, VisualTileSource};

use super::{Placement, architecture_box, native_source_surface_patches, native_source_swatches};

pub(super) fn local(map: &str, source: &VisualTileSource) -> Option<(u8, u8, usize, usize)> {
    if !crate::train_station::supports_map(map)
        || source.tileset_id.as_ref() != "train_station"
        || source.subtile_column >= 4
        || source.subtile_row >= 2
    {
        return None;
    }
    match source.metatile_id {
        0x24 if source.tile_index == 0x30 => {
            Some((source.subtile_column, source.subtile_row, 4, 2))
        }
        0x25 if source.tile_index
            == [[0x4e, 0x4f], [0x5e, 0x5f]][usize::from(source.subtile_row)]
                [usize::from(source.subtile_column % 2)] =>
        {
            Some((source.subtile_column % 2, source.subtile_row, 2, 2))
        }
        _ => None,
    }
}

pub(super) fn matches(map: &str, p: &Placement, cells: &[&VisualTile], g: &GridGeometry) -> bool {
    if g.width.checked_mul(g.height) != Some(cells.len())
        || p.column + p.width > g.width
        || p.row + p.height > g.height
        || p.height != 2
        || !matches!(p.width, 2 | 4)
    {
        return false;
    }
    let first = &cells[p.row * g.width + p.column].source;
    let kind = if first.metatile_id == 0x24 {
        ModelKind::WallClinical
    } else {
        ModelKind::PictureFrame
    };
    p.kind == kind
        && p.indices(g.width).enumerate().all(|(index, cell)| {
            let source = &cells[cell].source;
            source.metatile_id == first.metatile_id
                && source.subtile_column == first.subtile_column + (index % p.width) as u8
                && local(map, source)
                    == Some(((index % p.width) as u8, (index / p.width) as u8, p.width, 2))
        })
}

pub(super) fn append_wall(
    out: &mut SurfaceMeshData,
    g: &GridGeometry,
    p: &Placement,
    bounds: [f32; 4],
) {
    let [west, east, north, south] = bounds;
    let height = p.height_pixels * g.tile_height / 8.0;
    let [blue, mortar] = native_source_swatches(p, g, [[2, 2], [1, 0]]);
    let first = out.positions.len();
    let indices = out.indices.len();
    architecture_box(out, [west, east, 0.0, height, north, south], [1.0; 3]);
    out.uvs[first..].fill(blue);
    let faces = out.indices.split_off(indices);
    out.indices.extend(
        faces
            .as_chunks::<3>()
            .0
            .iter()
            .filter(|triangle| out.normals[triangle[0] as usize][2] < 0.5)
            .flatten()
            .copied(),
    );
    native_source_surface_patches(
        out,
        &[0.0, 0.025, 0.25, 0.275, 0.5, 0.525, 0.75, 0.775, 1.0],
        &[0.0, 0.04, 0.25, 0.29, 0.5, 0.54, 0.75, 0.79, 1.0],
        |x, y| [west + (east - west) * x, height * (1.0 - y), south],
        |x, y| {
            if (x * 4.0).fract() < 0.1 || (y * 4.0).fract() < 0.16 {
                mortar
            } else {
                blue
            }
        },
        |_, _| 1.0,
        true,
    );
}

#[cfg(test)]
mod tests {
    use bevy::prelude::Vec3;

    use super::{GridGeometry, ModelKind, Placement, SurfaceMeshData, append_wall};

    #[test]
    fn waiting_wall_has_an_outward_front_and_closed_native_bounds() {
        let geometry = GridGeometry {
            width: 4,
            height: 2,
            tile_width: 8.0,
            tile_height: 8.0,
            origin_x: 0.0,
            origin_z: 0.0,
        };
        let placement = Placement {
            kind: ModelKind::WallClinical,
            column: 0,
            row: 0,
            width: 4,
            height: 2,
            ground: 0,
            depth_pixels: 3.0,
            height_pixels: 16.0,
            base_pixels: 0.0,
            footing_pixels: None,
        };
        let mut mesh = SurfaceMeshData::default();
        append_wall(&mut mesh, &geometry, &placement, [0.0, 32.0, 0.0, 3.0]);
        let mut front_faces = 0;
        for triangle in mesh.indices.as_chunks::<3>().0 {
            let points = [triangle[0], triangle[1], triangle[2]]
                .map(|index| Vec3::from_array(mesh.positions[index as usize]));
            if points.iter().all(|point| point.z == 3.0) {
                let normal = (points[1] - points[0])
                    .cross(points[2] - points[0])
                    .normalize();
                assert!(
                    normal.z > 0.99,
                    "the visible masonry front must face the public room"
                );
                front_faces += 1;
            }
        }
        assert!(front_faces > 4);
        assert!(
            mesh.positions
                .iter()
                .all(|[x, y, z]| (0.0..=32.0).contains(x)
                    && (0.0..=16.0).contains(y)
                    && (0.0..=3.0).contains(z))
        );
    }
}
