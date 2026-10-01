use super::*;
use bevy::render::mesh::VertexAttributeValues;

fn triangle(offset: f32) -> SurfaceMeshData {
    SurfaceMeshData {
        positions: vec![
            [offset, 0.0, 0.0],
            [offset + 1.0, 0.0, 0.0],
            [offset, 1.0, 0.0],
        ],
        normals: vec![[0.0, 0.0, 1.0]; 3],
        uvs: vec![[0.25, 0.5]; 3],
        colors: vec![[1.0; 4]; 3],
        indices: vec![0, 1, 2],
        ..Default::default()
    }
}

#[test]
fn indoor_cutaway_marks_only_selected_vertices_without_changing_geometry() {
    let original = triangle(0.0);
    let mut data = original.clone();
    data.cutaway_ranges.push(0..3);
    let ordinary = original.into_mesh();
    let marked = data.into_mesh();
    assert!(ordinary.attribute(Mesh::ATTRIBUTE_UV_1).is_none());
    assert!(matches!(marked.attribute(Mesh::ATTRIBUTE_UV_1),
        Some(VertexAttributeValues::Float32x2(mask)) if mask == &vec![[1.0, 0.0]; 3]));
    for attribute in [
        Mesh::ATTRIBUTE_POSITION,
        Mesh::ATTRIBUTE_NORMAL,
        Mesh::ATTRIBUTE_UV_0,
        Mesh::ATTRIBUTE_COLOR,
    ] {
        assert_eq!(
            marked.attribute(attribute.id).unwrap().get_bytes(),
            ordinary.attribute(attribute.id).unwrap().get_bytes()
        );
    }
    assert_eq!(
        marked.indices().unwrap().iter().collect::<Vec<_>>(),
        ordinary.indices().unwrap().iter().collect::<Vec<_>>()
    );
}

#[test]
fn indoor_cutaway_ranges_survive_merged_surfaces_without_claiming_neighbors() {
    let mut terrain = TerrainMeshData::default();
    terrain.solid = triangle(0.0);
    terrain.animated_solid = triangle(2.0);
    terrain.animated_solid.cutaway_ranges.push(0..3);
    let (_, solid) = terrain.into_meshes();
    let Some(VertexAttributeValues::Float32x2(mask)) = solid.attribute(Mesh::ATTRIBUTE_UV_1) else {
        panic!("marked surface lost its mask during merge");
    };
    assert_eq!(
        mask,
        &vec![
            [0.0, 0.0],
            [0.0, 0.0],
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 0.0],
            [1.0, 0.0]
        ]
    );
}
