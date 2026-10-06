use super::*;

fn corners(
    surface: &SurfaceMeshData,
    indices: impl Iterator<Item = u32>,
) -> Vec<([f32; 3], [f32; 3], [f32; 2], [f32; 4], bool)> {
    indices
        .map(|index| {
            let i = index as usize;
            (
                surface.positions[i],
                surface.normals[i],
                surface.uvs[i],
                surface.colors[i],
                surface
                    .cutaway_ranges
                    .iter()
                    .any(|range| range.contains(&i)),
            )
        })
        .collect()
}

#[test]
fn reveal_partition_keeps_the_complete_public_mesh_and_every_indexed_corner() {
    let mut terrain = TerrainMeshData::default();
    // An ordinary unmarked prop before the walls exercises eligibility remapping.
    append_solid_quad(
        &mut terrain.solid,
        [
            [-4.0, 0.0, -4.0],
            [-4.0, 0.0, -3.0],
            [-3.0, 0.0, -3.0],
            [-3.0, 0.0, -4.0],
        ],
        [0.0, 1.0, 0.0],
        [1.0; 4],
    );
    for (exposed, covered, bounds) in [
        (0, 15, [0.0, 8.0, 0.0, 8.0]),
        (5, 10, [8.0, 16.0, 0.0, 8.0]),
    ] {
        let start = terrain.solid.indices.len() / 3;
        let vertex_start = terrain.solid.positions.len();
        crate::dungeon_models::gym_model(4 + usize::from(exposed)).append(
            &mut terrain.solid,
            bounds,
            0.0,
            12.0,
        );
        terrain
            .solid
            .cutaway_ranges
            .push(vertex_start..terrain.solid.positions.len());
        terrain.reveal_join_batches.push(
            crate::dungeon_models::gym_wall_join_triangles(exposed, covered)
                .iter()
                .map(|&ordinal| start + ordinal)
                .collect(),
        );
    }
    let original = terrain.solid.clone();
    let ordinals = terrain.reveal_join_batches.clone();
    // The ordinary public conversion is still the complete original mesh.
    let expected_mesh = original.clone().into_mesh();
    let (_, public_mesh) = terrain.clone().into_meshes();
    for attribute in [
        Mesh::ATTRIBUTE_POSITION,
        Mesh::ATTRIBUTE_NORMAL,
        Mesh::ATTRIBUTE_UV_0,
        Mesh::ATTRIBUTE_UV_1,
        Mesh::ATTRIBUTE_COLOR,
    ] {
        assert_eq!(
            expected_mesh.attribute(attribute.id).unwrap().get_bytes(),
            public_mesh.attribute(attribute.id).unwrap().get_bytes()
        );
    }
    assert_eq!(
        expected_mesh.indices().unwrap().iter().collect::<Vec<_>>(),
        public_mesh.indices().unwrap().iter().collect::<Vec<_>>()
    );

    let batches = terrain.take_reveal_join_batches();
    assert_eq!(batches.len(), ordinals.len());
    let mut remaining = vec![true; original.indices.len() / 3];
    for (batch, ordinals) in batches.iter().zip(ordinals) {
        for &ordinal in &ordinals {
            assert!(remaining[ordinal], "a triangle was assigned twice");
            remaining[ordinal] = false;
        }
        let indices = ordinals.into_iter().flat_map(|ordinal| {
            original.indices[ordinal * 3..ordinal * 3 + 3]
                .iter()
                .copied()
        });
        assert_eq!(
            corners(batch, batch.indices.iter().copied()),
            corners(&original, indices)
        );
    }
    let retained_indices = original
        .indices
        .chunks_exact(3)
        .zip(remaining)
        .filter(|(_, retain)| *retain)
        .flat_map(|(tri, _)| tri.iter().copied());
    assert_eq!(
        corners(&terrain.solid, terrain.solid.indices.iter().copied()),
        corners(&original, retained_indices)
    );
    assert_eq!(
        terrain.solid.indices.len() + batches.iter().map(|b| b.indices.len()).sum::<usize>(),
        original.indices.len()
    );
    let current = terrain.solid.clone();
    assert!(terrain.take_reveal_join_batches().is_empty());
    assert_eq!(
        terrain.solid, current,
        "runtime partition must happen only once"
    );
}
