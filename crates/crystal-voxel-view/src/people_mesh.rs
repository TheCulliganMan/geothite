//! Remove duplicate attribute records produced by clothing cuts.
//! Geometric coincidences alone never merge a UV, normal or skin seam.
use crate::mesh::SurfaceMeshData;
use std::collections::HashMap;

fn key(mesh: &SurfaceMeshData, ids: &[[u16; 4]], weights: &[[f32; 4]], i: usize) -> [u32; 21] {
    let mut key = [0; 21];
    key[..3].copy_from_slice(&mesh.positions[i].map(f32::to_bits));
    key[3..6].copy_from_slice(&mesh.normals[i].map(f32::to_bits));
    key[6..10].copy_from_slice(&mesh.colors[i].map(f32::to_bits));
    key[10..12].copy_from_slice(&mesh.uvs[i].map(f32::to_bits));
    key[12..16].copy_from_slice(&ids[i].map(u32::from));
    key[16..20].copy_from_slice(&weights[i].map(f32::to_bits));
    key[20] = u32::from(mesh.cutaway_ranges.iter().any(|range| range.contains(&i)));
    key
}

pub(crate) fn compact(
    mesh: &mut SurfaceMeshData,
    ids: &mut Vec<[u16; 4]>,
    weights: &mut Vec<[f32; 4]>,
) {
    let original = std::mem::take(mesh);
    let old_ids = std::mem::take(ids);
    let old_weights = std::mem::take(weights);
    let mut records = HashMap::<[u32; 21], u32>::new();
    for &index in &original.indices {
        let i = index as usize;
        let record = key(&original, &old_ids, &old_weights, i);
        let mapped = *records.entry(record).or_insert_with(|| {
            let mapped = mesh.positions.len() as u32;
            mesh.positions.push(original.positions[i]);
            mesh.normals.push(original.normals[i]);
            mesh.colors.push(original.colors[i]);
            mesh.uvs.push(original.uvs[i]);
            ids.push(old_ids[i]);
            weights.push(old_weights[i]);
            if record[20] != 0 {
                let i = mapped as usize;
                if let Some(last) = mesh
                    .cutaway_ranges
                    .last_mut()
                    .filter(|range| range.end == i)
                {
                    last.end = i + 1;
                } else {
                    mesh.cutaway_ranges.push(i..i + 1);
                }
            }
            mapped
        });
        #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
        assert_eq!(
            record,
            key(mesh, ids, weights, mapped as usize),
            "compaction changed a triangle corner"
        );
        mesh.indices.push(mapped);
    }
    assert_eq!(mesh.indices.len(), original.indices.len());
}

#[cfg(test)]
mod tests {
    #[test]
    fn duplicate_records_merge_without_merging_uv_normal_or_skin_seams() {
        let mut mesh = crate::mesh::SurfaceMeshData {
            positions: vec![[1., 2., 3.]; 6],
            normals: vec![[0., 0., 1.]; 6],
            colors: vec![[1.; 4]; 6],
            uvs: vec![[0.; 2]; 6],
            indices: vec![0, 1, 2, 2, 3, 4, 0, 5, 5],
            cutaway_ranges: vec![4..6],
        };
        mesh.uvs[2] = [1., 0.];
        mesh.normals[3] = [0., 1., 0.];
        let mut ids = vec![[2, 0, 0, 0]; 6];
        ids[4] = [3, 0, 0, 0];
        let mut weights = vec![[1., 0., 0., 0.]; 6];
        super::compact(&mut mesh, &mut ids, &mut weights);
        assert_eq!(mesh.positions.len(), 5);
        assert_eq!(mesh.indices, vec![0, 0, 1, 1, 2, 3, 0, 4, 4]);
        assert_eq!(mesh.uvs[1], [1., 0.]);
        assert_eq!(mesh.normals[2], [0., 1., 0.]);
        assert_eq!(ids[3], [3, 0, 0, 0]);
        assert_eq!(mesh.cutaway_ranges, vec![3..5]);
    }
}
