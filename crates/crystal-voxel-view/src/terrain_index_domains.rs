//! Dense spatial buckets retain their triangles in smaller GPU upload domains.
use crate::mesh::{SurfaceMeshData, select_surface_indices};

// Together these bounds fit below a 4 MiB upload allowance, even if a browser
// widens indices for mapped-buffer alignment. They do not limit scene content.
const MAX_VERTICES: usize = u16::MAX as usize + 1;
const MAX_INDICES: usize = MAX_VERTICES * 3;

pub(super) fn split(source: SurfaceMeshData) -> Vec<SurfaceMeshData> {
    if (source.positions.len() <= MAX_VERTICES && source.indices.len() <= MAX_INDICES)
        || !crate::terrain_batches::valid_topology(&source)
    {
        return vec![source];
    }
    let _trace = crate::loading_trace::span(
        "region.split_index_domains",
        || serde_json::json!({"vertices": source.positions.len(), "indices": source.indices.len()}),
    );
    let mut seen = vec![0_usize; source.positions.len()];
    let mut ranges = Vec::new();
    let (mut epoch, mut unique, mut start) = (1, 0, 0);
    let (triangles, _) = source.indices.as_chunks::<3>();
    for (triangle, indices) in triangles.iter().enumerate() {
        let offset = triangle * 3;
        // Repeated indices within a degenerate triangle can conservatively
        // overestimate this addition, but never enlarge the resulting domain.
        let added = indices
            .iter()
            .filter(|&&index| seen[index as usize] != epoch)
            .count();
        if offset > start && (unique + added > MAX_VERTICES || offset + 3 - start > MAX_INDICES) {
            ranges.push(start..offset);
            start = offset;
            unique = 0;
            epoch += 1;
        }
        for &index in indices {
            if seen[index as usize] != epoch {
                seen[index as usize] = epoch;
                unique += 1;
            }
        }
    }
    if start < source.indices.len() {
        ranges.push(start..source.indices.len());
    }
    let mut marked = vec![false; source.positions.len()];
    for range in &source.cutaway_ranges {
        marked[range.clone()].fill(true);
    }
    let mut remap = vec![u32::MAX; source.positions.len()];
    ranges
        .into_iter()
        .map(|range| {
            select_surface_indices(
                &source,
                &marked,
                &mut remap,
                source.indices[range].iter().copied(),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{MAX_INDICES, MAX_VERTICES, split};
    use crate::mesh::SurfaceMeshData;
    use bevy::mesh::Indices;

    #[test]
    fn dense_buckets_preserve_every_triangle_attribute_and_cutaway_owner() {
        let count = 90_000;
        let source = SurfaceMeshData {
            positions: (0..count).map(|i| [i as f32, 2., 3.]).collect(),
            normals: vec![[0., 1., 0.]; count],
            uvs: (0..count).map(|i| [i as f32 * 0.25, -40000.]).collect(),
            colors: vec![[0.3, 0.4, 0.5, 1.]; count],
            indices: (0..count as u32).collect(),
            cutaway_ranges: vec![65_530..65_540, 65_540..65_550],
        };
        let chunks = split(source.clone());
        assert!(chunks.len() > 1);
        let mut original = source.indices.iter();
        for chunk in chunks {
            assert!(chunk.positions.len() <= MAX_VERTICES);
            assert!(chunk.indices.len() <= MAX_INDICES);
            for &index in &chunk.indices {
                let old = *original.next().expect("no additional triangles") as usize;
                let index = index as usize;
                assert_eq!(chunk.positions[index], source.positions[old]);
                assert_eq!(chunk.normals[index], source.normals[old]);
                assert_eq!(chunk.uvs[index], source.uvs[old]);
                assert_eq!(chunk.colors[index], source.colors[old]);
                assert_eq!(
                    chunk
                        .cutaway_ranges
                        .iter()
                        .any(|range| range.contains(&index)),
                    source
                        .cutaway_ranges
                        .iter()
                        .any(|range| range.contains(&old))
                );
            }
            let mesh = chunk.into_compact_mesh();
            let Some(Indices::U16(indices)) = mesh.indices() else {
                panic!("each upload domain must fit lossless 16-bit indices");
            };
            assert!(mesh.get_vertex_buffer_size() + indices.len() * 4 < 4 * 1024 * 1024);
        }
        assert!(original.next().is_none(), "no missing triangles");
    }

    #[test]
    fn repeated_triangle_indices_cannot_create_an_oversized_index_upload() {
        let mut source = SurfaceMeshData {
            positions: vec![[0., 0., 0.], [1., 0., 0.], [0., 0., 1.]],
            normals: vec![[0., 1., 0.]; 3],
            uvs: vec![[0., 0.]; 3],
            colors: vec![[1.; 4]; 3],
            ..Default::default()
        };
        source.indices = (0..MAX_INDICES / 3 + 1).flat_map(|_| [0, 1, 2]).collect();
        let chunks = split(source);
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].indices.len(), MAX_INDICES);
        assert_eq!(chunks[1].indices, [0, 1, 2]);
    }
}
