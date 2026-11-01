//! Preserve authored building geometry across the source terrace field.
use super::{SurfaceMeshData, terraces::Field};
use bevy::prelude::Vec2;
use std::ops::Range;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Building {
    pub vertices: Range<usize>,
    pub doorway: Vec2,
}

pub(super) fn owners(vertices: usize, buildings: &[Building]) -> Vec<bool> {
    let mut owned = vec![false; vertices];
    for building in buildings {
        assert!(building.vertices.end <= vertices);
        owned[building.vertices.clone()].fill(true);
    }
    owned
}

pub(super) fn lift(surface: &mut SurfaceMeshData, buildings: &[Building], field: &Field<'_>) {
    for building in buildings {
        let height = field.height(building.doorway);
        for point in &mut surface.positions[building.vertices.clone()] {
            point[1] += height;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::{GridGeometry, TerrainMeshData};
    use bevy::prelude::IVec2;
    use crystal_render_api::VisualJumpLedge;

    #[test]
    fn terrace_grade_preserves_authored_roof_and_doorway_geometry() {
        let frame = crate::mesh::tests::frame(12, 12,
            vec![crate::mesh::tests::source_with_tile(0, 0, 0, 5); 144]);
        let cells = frame.tiles.iter().collect::<Vec<_>>();
        let geometry = GridGeometry { width:12, height:12, tile_width:8., tile_height:8.,
            origin_x:0., origin_z:0. };
        let edges = [VisualJumpLedge { start:IVec2::new(4,4), end:IVec2::new(8,4),
            descent:IVec2::Y }];
        let field = Field::new(&edges, &geometry, [0,0]);
        let mut mesh = TerrainMeshData { footing_heights:vec![0.;144], ..Default::default() };
        let doorway = Vec2::new(48., 28.);
        let height = field.height(doorway);
        assert!(height > 0.1);
        // This wide plot crosses the real grade beside a finite ledge.
        assert!((field.height(Vec2::new(8.,28.))-height).abs()>0.01);
        crate::new_bark_models::model(crate::new_bark_models::ModelKind::VioletGym)
            .append_fitted(&mut mesh.solid, [8.,88.,8.,88.], 0.,16.,Some(48.));
        let original = mesh.solid.clone();
        mesh.rigid_buildings.push(Building {vertices:0..mesh.solid.positions.len(),doorway});
        super::super::terraces::finish(&mut mesh,&cells,&geometry,[0,0],&edges);
        assert_eq!(&mesh.solid.indices[..original.indices.len()], &original.indices);
        assert_eq!(&mesh.solid.normals[..original.normals.len()], &original.normals);
        assert_eq!(&mesh.solid.colors[..original.colors.len()], &original.colors);
        for (before, after) in original.positions.iter().zip(&mesh.solid.positions) {
            assert_eq!(before[0],after[0]);
            assert_eq!(before[2],after[2]);
            assert!((after[1]-before[1]-height).abs()<0.00001,
                "roof decks and tiles must share the doorway's rigid support shift");
        }
        // The source ground still receives its real graded support independently.
        for (i,&actual) in mesh.footing_heights.iter().enumerate() {
            let expected=field.height(Vec2::new((i%12) as f32*8.+4.,(i/12) as f32*8.+4.));
            assert!((actual-expected).abs()<0.00001);
        }
    }
}
