//! Give sea-level outdoor banks physical depth without lifting dry land.
//! The native profile stays useful to source adapters; this final fit moves
//! the completed liquid, its rooted scenery and resolved actor support together.
use super::*;
use std::collections::BTreeMap;

// A quarter tile of exposed bank, rather than the native half-pixel sliver.
const WATERLINE: f32 = -2.;

pub(super) fn level(tile_height: f32) -> f32 {
    WATERLINE * tile_height / SOURCE_TILE_HEIGHT
}

fn cell_at(g: &GridGeometry, p: [f32; 3]) -> Option<usize> {
    let x = ((p[0] - g.origin_x) / g.tile_width).floor() as isize;
    let z = ((p[2] - g.origin_z) / g.tile_height).floor() as isize;
    (x >= 0 && z >= 0 && x < g.width as isize && z < g.height as isize)
        .then(|| z as usize * g.width + x as usize)
}

fn fit_surface(
    surface: &mut SurfaceMeshData,
    g: &GridGeometry,
    buoys: &[bool],
    heights: &[f32],
    native: f32,
    drop: f32,
) {
    let has_buoys = buoys.iter().any(|&v| v);
    let mut movement = Vec::with_capacity(surface.positions.len());
    for (p, uv) in surface.positions.iter_mut().zip(&surface.uvs) {
        let cell = (uv[0] == super::shore_ecology::ROOTED_WATER_UV || has_buoys)
            .then(|| cell_at(g, *p))
            .flatten();
        let rooted =
            uv[0] == super::shore_ecology::ROOTED_WATER_UV || cell.is_some_and(|i| buoys[i]);
        if rooted
            && cell
                .and_then(|i| heights.get(i))
                .is_some_and(|&h| h > 0.0001)
        {
            movement.push(0);
            continue;
        }
        let fraction = if rooted {
            1.
        } else {
            (p[1] / native).clamp(0., 1.)
        };
        p[1] -= drop * fraction;
        movement.push(if fraction == 0. {
            0
        } else if fraction == 1. {
            1
        } else {
            2
        });
    }
    // Rebuild only faces whose shape changed. Rigidly lowered rocks, floats
    // and plants retain their original normals and sculpted proportions.
    // Include the previous normal in the weld key to preserve hard creases.
    if movement.iter().all(|&m| m == 0) || movement.iter().all(|&m| m == 1) {
        return;
    }
    let mut normals = BTreeMap::<[u32; 6], Vec3>::new();
    let mut changed = vec![false; surface.positions.len()];
    let key = |i: usize| {
        let p = surface.positions[i].map(f32::to_bits);
        let n = surface.normals[i].map(f32::to_bits);
        [p[0], p[1], p[2], n[0], n[1], n[2]]
    };
    for tri in surface.indices.chunks_exact(3) {
        let ids = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
        let flags = ids.map(|i| movement[i]);
        if flags == [0; 3] || flags == [1; 3] {
            continue;
        }
        let p = ids.map(|i| Vec3::from_array(surface.positions[i]));
        let n = (p[1] - p[0]).cross(p[2] - p[0]);
        for i in ids {
            *normals.entry(key(i)).or_default() += n;
            changed[i] = true;
        }
    }
    let replacements = changed
        .iter()
        .enumerate()
        .filter(|(_, changed)| **changed)
        .map(|(i, _)| (i, normals[&key(i)].normalize_or_zero().to_array()))
        .collect::<Vec<_>>();
    for (i, normal) in replacements {
        if Vec3::from_array(normal).length_squared() > 0.5 {
            surface.normals[i] = normal;
        }
    }
}

pub(super) fn finish(
    mesh: &mut TerrainMeshData,
    map: &str,
    cells: &[&VisualTile],
    g: &GridGeometry,
) {
    if !crate::cave_water::outdoor(map)
        || g.width == 0
        || g.height == 0
        || cells.len() != g.width * g.height
        || mesh.authored_cells.len() != cells.len()
        || mesh.footing_heights.len() != cells.len()
        || !cells
            .iter()
            .any(|t| matches!(shape_for_source(&t.source), CellShape::Water))
    {
        return;
    }
    let unit = g.tile_height / SOURCE_TILE_HEIGHT;
    let native = crate::profile::WATER_HEIGHT * unit;
    let drop = (crate::profile::WATER_HEIGHT - WATERLINE) * unit;
    let buoys = mesh
        .authored_cells
        .iter()
        .map(|c| *c == Some(super::buoy_course::LABEL))
        .collect::<Vec<_>>();
    for surface in [
        &mut mesh.textured,
        &mut mesh.solid,
        &mut mesh.animated_textured,
        &mut mesh.animated_solid,
    ] {
        fit_surface(surface, g, &buoys, &mesh.footing_heights, native, drop);
    }
    if let Some(background) = mesh.background.as_mut()
        && new_bark::ground_material(&background.source) == Some(new_bark::GroundMaterial::Water)
    {
        fit_surface(
            &mut background.mesh,
            g,
            &buoys,
            &mesh.footing_heights,
            native,
            drop,
        );
    }
    for (i, height) in mesh.footing_heights.iter_mut().enumerate() {
        let source_water = matches!(shape_for_source(&cells[i].source), CellShape::Water);
        let known = matches!(
            mesh.authored_cells[i],
            None | Some("world-exterior/kanto_boundary_shore" | "outdoor/buoy-course")
        );
        // Actor roots keep their existing immersion above the liquid. Raised
        // inland pools and custom supported objects retain their own datum.
        if known
            && (source_water
                || buoys[i]
                || mesh.authored_cells[i] == Some("world-exterior/kanto_boundary_shore"))
            && *height <= 0.0001 * unit
            && *height >= native - 0.0001 * unit
        {
            *height -= drop;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{frame, source_with_tile};
    use super::*;

    fn fixture(scale: f32) -> (VisualWorldFrame, GridGeometry, TerrainMeshData) {
        let mut f = frame(
            3,
            1,
            vec![
                source_with_tile(1, 0, 0, 5),
                source_with_tile(0x35, 0, 0, 0x14),
                source_with_tile(0x35, 1, 0, 0x14),
            ],
        );
        f.map_id = "Route40".into();
        f.tile_size *= scale;
        let g = GridGeometry {
            width: 3,
            height: 1,
            tile_width: 8. * scale,
            tile_height: 8. * scale,
            origin_x: 0.,
            origin_z: 0.,
        };
        let mut mesh = TerrainMeshData {
            footing_heights: vec![0.; 3],
            authored_cells: vec![None; 3],
            ..Default::default()
        };
        append_top(
            &mut mesh.textured,
            [0., 8. * scale, 0., 8. * scale],
            0.,
            g.uv(0, 0),
        );
        append_textured_shoreline(
            &mut mesh.textured,
            &g,
            0,
            0,
            Direction::East,
            [0., 8. * scale, 0., 8. * scale],
            crate::profile::WATER_HEIGHT * scale,
            0.,
        );
        append_top(
            &mut mesh.animated_textured,
            [8. * scale, 24. * scale, 0., 8. * scale],
            crate::profile::WATER_HEIGHT * scale,
            g.uv(1, 0),
        );
        (f, g, mesh)
    }

    #[test]
    fn actual_bank_closes_to_deeper_liquid_with_source_ground_and_actor_immersion_intact() {
        for scale in [0.5, 1., 2.] {
            let (f, g, mut mesh) = fixture(scale);
            let before = mesh.clone();
            let cells = f.tiles.iter().collect::<Vec<_>>();
            finish(&mut mesh, "Route40", &cells, &g);
            assert_eq!(
                &mesh.textured.positions[..4],
                &before.textured.positions[..4]
            );
            assert_eq!(mesh.textured.indices, before.textured.indices);
            assert_eq!(mesh.textured.uvs, before.textured.uvs);
            assert_eq!(mesh.textured.colors, before.textured.colors);
            assert_eq!(mesh.authored_cells, before.authored_cells);
            assert!(
                mesh.animated_textured
                    .positions
                    .iter()
                    .all(|p| p[1] == WATERLINE * scale)
            );
            assert!(
                mesh.textured
                    .positions
                    .iter()
                    .any(|p| p[1] == WATERLINE * scale)
            );
            assert_eq!(mesh.footing_heights, vec![0., -1.5 * scale, -1.5 * scale]);
            assert_eq!(
                crate::footing::resolved_footing_height(&f, f.center, &mesh.footing_heights),
                Some(-1.5 * scale)
            );
            assert_eq!(
                mesh.footing_heights[1] - mesh.animated_textured.positions[0][1],
                0.5 * scale
            );
            for tri in mesh.textured.indices.chunks_exact(3) {
                let ids = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
                let p = ids.map(|i| Vec3::from_array(mesh.textured.positions[i]));
                let n = ids
                    .into_iter()
                    .map(|i| Vec3::from_array(mesh.textured.normals[i]))
                    .sum::<Vec3>();
                assert!((p[1] - p[0]).cross(p[2] - p[0]).dot(n) > 0.);
            }
        }
    }

    #[test]
    fn rooted_scenery_and_native_floats_move_rigidly_without_repainting() {
        let (f, g, mut mesh) = fixture(1.);
        let cells = f.tiles.iter().collect::<Vec<_>>();
        mesh.authored_cells[2] = Some(super::super::buoy_course::LABEL);
        append_quad_colors(
            &mut mesh.solid,
            [[8., -0.5, 0.], [8., 3., 0.], [12., 3., 0.], [12., -0.5, 0.]],
            [0., 0., -1.],
            [[super::super::shore_ecology::ROOTED_WATER_UV, 0.]; 4],
            [[0.4, 0.5, 0.6, 1.]; 4],
        );
        append_quad_colors(
            &mut mesh.solid,
            [
                [17., -0.5, 2.],
                [17., 2., 2.],
                [20., 2., 2.],
                [20., -0.5, 2.],
            ],
            [0., 0., -1.],
            [[0., 0.]; 4],
            [[0.4, 0.5, 0.6, 1.]; 4],
        );
        let before = mesh.solid.clone();
        finish(&mut mesh, "Route40", &cells, &g);
        for (after, before) in mesh.solid.positions.iter().zip(&before.positions) {
            assert_eq!(after, &[before[0], before[1] - 1.5, before[2]]);
        }
        assert_eq!(mesh.solid.normals, before.normals);
        assert_eq!(mesh.solid.colors, before.colors);
        assert_eq!(mesh.solid.uvs, before.uvs);
    }

    #[test]
    fn open_sea_continuation_and_retained_water_share_the_completed_datum() {
        let (f, g, mut mesh) = fixture(1.);
        let mut apron = SurfaceMeshData::default();
        append_top(
            &mut apron,
            [-100., 0., -100., 100.],
            crate::profile::WATER_HEIGHT,
            (0., 1., 0., 1.),
        );
        super::super::background::finish_water_apron(&mut mesh, apron);
        finish(
            &mut mesh,
            "Route40",
            &f.tiles.iter().collect::<Vec<_>>(),
            &g,
        );
        assert!(mesh.background.is_none());
        assert!(
            mesh.textured
                .positions
                .iter()
                .filter(|p| p[0] < 0.)
                .all(|p| p[1] == WATERLINE)
        );
        assert!(
            mesh.animated_textured
                .positions
                .iter()
                .all(|p| p[1] == WATERLINE)
        );
    }

    #[test]
    fn interiors_dry_maps_raised_pools_and_custom_support_keep_their_datums() {
        let (f, g, baseline) = fixture(1.);
        let cells = f.tiles.iter().collect::<Vec<_>>();
        for map in ["CeruleanGym", "DragonsDenB1F", "WillsRoom"] {
            let mut m = baseline.clone();
            finish(&mut m, map, &cells, &g);
            assert_eq!(m, baseline);
        }
        let mut m = baseline.clone();
        m.footing_heights[1] = 12.;
        m.authored_cells[2] = Some("custom:platform");
        append_quad_colors(
            &mut m.solid,
            [
                [9., 11.5, 1.],
                [9., 14., 1.],
                [12., 14., 1.],
                [12., 11.5, 1.],
            ],
            [0., 0., -1.],
            [[super::super::shore_ecology::ROOTED_WATER_UV, 0.]; 4],
            [[0.4, 0.5, 0.6, 1.]; 4],
        );
        let raised = m.solid.clone();
        finish(&mut m, "Route40", &cells, &g);
        assert_eq!(m.footing_heights, [0., 12., 0.]);
        assert_eq!(m.solid, raised);
        let mut dry = f.clone();
        for t in &mut dry.tiles {
            t.source = source_with_tile(1, 0, 0, 5);
        }
        let mut m = baseline.clone();
        finish(&mut m, "Route40", &dry.tiles.iter().collect::<Vec<_>>(), &g);
        assert_eq!(m, baseline);
    }
}
