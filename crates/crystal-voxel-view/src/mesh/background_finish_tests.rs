use super::super::{
    background,
    tests::{frame, source_with_tile},
};
use super::*;
use bevy::prelude::{IVec2, UVec2};
use crystal_render_api::VisualSourceRegion;

fn geometry(frame: &VisualWorldFrame) -> GridGeometry {
    GridGeometry {
        width: frame.grid_size.x as usize,
        height: frame.grid_size.y as usize,
        tile_width: frame.tile_size.x,
        tile_height: frame.tile_size.y,
        origin_x: -32.,
        origin_z: -40.,
    }
}

fn park_source() -> VisualTileSource {
    let mut source = source_with_tile(1, 0, 0, 0);
    source.tileset_id = "park".into();
    source
}

#[test]
fn park_background_pavers_finish_the_existing_apron_without_expanding_or_owning_it() {
    for map in ["NationalPark", "NationalParkBugContest"] {
        let mut frame = frame(40, 40, vec![park_source(); 1600]);
        frame.map_id = map.into();
        let unchanged = frame.clone();
        let cells = frame.tiles.iter().collect::<Vec<_>>();
        let g = geometry(&frame);
        let mut mesh = TerrainMeshData {
            tree_cache: Some(Default::default()),
            footing_heights: vec![0.; cells.len()],
            authored_cells: vec![None; cells.len()],
            ..Default::default()
        };
        let shapes = cells
            .iter()
            .map(|t| shape_for_source(&t.source))
            .collect::<Vec<_>>();
        assert!(
            background::append_repeating_background_apron(&mut mesh, &g, &cells, &shapes, map)
                .positions
                .is_empty()
        );
        let original = mesh.background.as_ref().unwrap().clone();
        let [x, z] = original.source_cell;
        assert_eq!(original.source, cells[z * g.width + x].source);
        assert_eq!(original.mesh.quad_count(), 4);
        append_top(
            &mut mesh.textured,
            g.bounds(20, 20).into(),
            0.,
            g.uv(20, 20),
        );
        let mut control = mesh.clone();
        control.background = None;
        polish_world_surfaces_from_frame(&mut control, &frame, &cells, &g, &shapes);
        polish_world_surfaces_from_frame(&mut mesh, &frame, &cells, &g, &shapes);
        assert!(
            mesh.background.is_none(),
            "known paving must leave the native texture domain"
        );
        let first = control.solid.positions.len();
        let index_first = control.solid.indices.len();
        assert_eq!(mesh.solid.positions[..first], control.solid.positions);
        assert_eq!(mesh.solid.normals[..first], control.solid.normals);
        assert_eq!(mesh.solid.uvs[..first], control.solid.uvs);
        assert_eq!(mesh.solid.colors[..first], control.solid.colors);
        assert_eq!(mesh.solid.indices[..index_first], control.solid.indices);
        assert_eq!(mesh.solid.positions[first..], original.mesh.positions);
        assert_eq!(mesh.solid.normals[first..], original.mesh.normals);
        assert_eq!(
            mesh.solid.indices[index_first..]
                .iter()
                .map(|i| i - first as u32)
                .collect::<Vec<_>>(),
            original.mesh.indices
        );
        for (p, uv) in mesh.solid.positions[first..]
            .iter()
            .zip(&mesh.solid.uvs[first..])
        {
            assert_eq!(
                *uv,
                [
                    66000. + (p[0] - g.origin_x) / g.tile_width,
                    -80000. + (p[2] - g.origin_z) / g.tile_height
                ]
            );
        }
        assert_eq!(mesh.textured, control.textured);
        assert_eq!(mesh.footing_heights, control.footing_heights);
        assert_eq!(mesh.authored_cells, control.authored_cells);
        assert_eq!(frame, unchanged);

        let mut unknown = TerrainMeshData {
            background: Some(original),
            ..Default::default()
        };
        unknown.background.as_mut().unwrap().source.tile_index = u16::MAX;
        let retained = unknown.background.clone();
        polish_world_surfaces_from_frame(&mut unknown, &frame, &cells, &g, &shapes);
        assert_eq!(
            unknown.background, retained,
            "unknown art retains its original material"
        );
        assert!(unknown.solid.positions.is_empty());
    }
}

#[test]
fn park_background_paving_requires_the_existing_outside_plane_and_real_source() {
    let frame = frame(4, 4, vec![park_source(); 16]);
    let cells = frame.tiles.iter().collect::<Vec<_>>();
    let g = geometry(&frame);
    let mut plane = SurfaceMeshData::default();
    append_top(
        &mut plane,
        [
            g.origin_x - g.tile_width,
            g.origin_x,
            g.origin_z,
            g.origin_z + g.tile_height,
        ],
        background::BACKGROUND_HEIGHT,
        g.uv(0, 0),
    );
    let p = &plane.positions;
    let source = &cells[0].source;
    assert_eq!(
        surface_material("NationalPark", p, [0., 1., 0.], source, &cells, &g, &[]),
        Some(GroundMaterial::Pavers)
    );
    let raised = p
        .iter()
        .map(|p| [p[0], p[1] + 0.1, p[2]])
        .collect::<Vec<_>>();
    assert_eq!(
        surface_material(
            "NationalPark",
            &raised,
            [0., 1., 0.],
            source,
            &cells,
            &g,
            &[]
        ),
        None
    );
    let inside = p
        .iter()
        .map(|p| [p[0] + g.tile_width, p[1], p[2]])
        .collect::<Vec<_>>();
    assert_eq!(
        surface_material(
            "NationalPark",
            &inside,
            [0., 1., 0.],
            source,
            &cells,
            &g,
            &[]
        ),
        None
    );
    assert_eq!(
        surface_material("NationalPark", p, [1., 0., 0.], source, &cells, &g, &[]),
        None
    );
    let mut unknown = source.clone();
    unknown.tile_index = u16::MAX;
    assert_eq!(
        surface_material("NationalPark", p, [0., 1., 0.], &unknown, &cells, &g, &[]),
        None
    );
}

#[test]
fn background_donor_keeps_palette_and_grain_origin_across_map_handoff_in_both_build_paths() {
    let source = source_with_tile(1, 0, 0, 6);
    let mut before = frame(20, 4, vec![source; 80]);
    before.map_id = "Route29".into();
    before.grid_origin = IVec2::new(72, 0);
    before.source_regions = vec![
        VisualSourceRegion {
            map_id: "Route29".into(),
            origin: IVec2::ZERO,
            size: UVec2::new(80, 40),
        },
        VisualSourceRegion {
            map_id: "NewBarkTown".into(),
            origin: IVec2::new(80, 0),
            size: UVec2::new(40, 40),
        },
    ]
    .into();
    let mut after = before.clone();
    after.map_id = "NewBarkTown".into();
    after.grid_origin -= IVec2::new(80, 0);
    after.source_regions = vec![
        VisualSourceRegion {
            map_id: "NewBarkTown".into(),
            origin: IVec2::ZERO,
            size: UVec2::new(40, 40),
        },
        VisualSourceRegion {
            map_id: "Route29".into(),
            origin: IVec2::new(-80, 0),
            size: UVec2::new(80, 40),
        },
    ]
    .into();
    let g = geometry(&before);
    let donor = [12, 0];
    let mut apron = SurfaceMeshData::default();
    append_top(
        &mut apron,
        [
            g.origin_x - g.tile_width,
            g.origin_x,
            g.origin_z,
            g.origin_z + g.tile_height,
        ],
        background::BACKGROUND_HEIGHT,
        g.uv(donor[0], donor[1]),
    );
    let render = |frame: &VisualWorldFrame, compact: bool| {
        let cells = frame.tiles.iter().collect::<Vec<_>>();
        assert_eq!(
            cells[0].source, cells[donor[0]].source,
            "art identity cannot identify the donor map"
        );
        let mut mesh = TerrainMeshData {
            footing_heights: vec![0.; cells.len()],
            authored_cells: vec![None; cells.len()],
            ..Default::default()
        };
        append_top(&mut mesh.textured, g.bounds(0, 0).into(), 0., g.uv(0, 0));
        if compact {
            mesh.background = Some(RepeatingBackground {
                mesh: apron.clone(),
                texture: cells[donor[0]].texture.clone(),
                source: cells[donor[0]].source.clone(),
                source_cell: donor,
            });
        } else {
            append_quad_colors(
                &mut mesh.textured,
                apron.positions[..].try_into().unwrap(),
                apron.normals[0],
                apron.uvs[..].try_into().unwrap(),
                apron.colors[..].try_into().unwrap(),
            );
        }
        polish_world_surfaces_from_frame(&mut mesh, frame, &cells, &g, &[]);
        assert!(mesh.background.is_none());
        let mut correct = GroundFinish::new("NewBarkTown", &cells, &g, [-8, 0]);
        correct.select_source_style("NewBarkTown", [-8, 0], false, true);
        let incorrect = GroundFinish::new("Route29", &cells, &g, [72, 0]);
        assert_ne!(
            correct.color(GroundMaterial::Path, apron.positions[0]),
            incorrect.color(GroundMaterial::Path, apron.positions[0])
        );
        let vertices = mesh
            .solid
            .positions
            .iter()
            .enumerate()
            .filter(|(_, p)| p[1] == background::BACKGROUND_HEIGHT)
            .map(|(i, p)| {
                assert_eq!(
                    mesh.solid.colors[i],
                    correct.color(GroundMaterial::Path, *p)
                );
                assert_eq!(
                    mesh.solid.uvs[i],
                    [
                        66000. + (p[0] - g.origin_x) / g.tile_width - 8.,
                        -60000. + (p[2] - g.origin_z) / g.tile_height
                    ]
                );
                (
                    *p,
                    mesh.solid.normals[i],
                    mesh.solid.uvs[i],
                    mesh.solid.colors[i],
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(vertices.len(), 4);
        assert_eq!(mesh.footing_heights, vec![0.; cells.len()]);
        assert_eq!(mesh.authored_cells, vec![None; cells.len()]);
        vertices
    };
    let expected = render(&before, false);
    for frame in [&before, &after] {
        for compact in [false, true] {
            assert_eq!(render(frame, compact), expected);
        }
    }
}
