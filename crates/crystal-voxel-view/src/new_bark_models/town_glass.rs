//! Pane coordinates come from connected authored glass, not the fitted world.
use crate::mesh::SurfaceMeshData;
use std::{collections::{BTreeMap, HashMap}, ops::Range};

pub(super) fn coordinates(mesh: &SurfaceMeshData, vertices: Range<usize>) -> Vec<(usize, [f32; 2])> {
    let mut parent: Vec<usize> = (0..vertices.len()).collect();
    fn root(parent: &mut [usize], mut i: usize) -> usize {
        while parent[i] != i {
            parent[i] = parent[parent[i]];
            i = parent[i];
        }
        i
    }
    fn join(parent: &mut [usize], a: usize, b: usize) {
        let a = root(parent, a);
        let b = root(parent, b);
        parent[b] = a;
    }
    // Flat-shaded exports duplicate positions at face-normal seams. Weld only
    // coincident vertices within this glass material; adjacent frames and
    // overlapping highlight strips are not part of the pane.
    let mut welded = HashMap::new();
    for i in vertices.clone() {
        let key = mesh.positions[i].map(|v| if v == 0. { 0 } else { v.to_bits() });
        let local = i - vertices.start;
        if let Some(&previous) = welded.get(&key) {
            join(&mut parent, previous, local);
        } else {
            welded.insert(key, local);
        }
    }
    for triangle in mesh.indices.chunks_exact(3) {
        if triangle.iter().all(|&i| vertices.contains(&(i as usize))) {
            let ids = [triangle[0], triangle[1], triangle[2]].map(|i| i as usize - vertices.start);
            join(&mut parent, ids[0], ids[1]);
            join(&mut parent, ids[0], ids[2]);
        }
    }
    let mut panes: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for i in vertices.clone() {
        panes.entry(root(&mut parent, i - vertices.start)).or_default().push(i);
    }
    let mut result = Vec::new();
    for pane in panes.values() {
        let mut low = [f32::INFINITY; 3];
        let mut high = [f32::NEG_INFINITY; 3];
        for &i in pane {
            for a in 0..3 {
                low[a] = low[a].min(mesh.positions[i][a]);
                high[a] = high[a].max(mesh.positions[i][a]);
            }
        }
        let span = std::array::from_fn::<_, 3, _>(|a| high[a] - low[a]);
        let horizontal = if span[0] > span[2] { 0 } else { 2 };
        // Decline degenerate or horizontal panes rather than inventing UVs.
        if span[horizontal] < 0.03 || span[1] < 0.08 { continue; }
        for &i in pane {
            let p = mesh.positions[i];
            result.push((i, [(p[horizontal] - low[horizontal]) / span[horizontal],
                crate::scenery_finish::TOWN_GLASS_BASE + (p[1] - low[1]) / span[1]]));
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disconnected_glass_panes_normalize_independently_across_flat_normal_seams() {
        let mut mesh = SurfaceMeshData::default();
        for (x, width, height) in [(0., 1., 1.), (4., 2., 3.)] {
            let corners = [[x,0.,0.], [x+width,0.,0.], [x+width,height,0.], [x,height,0.]];
            let center = [x+width*0.5,height*0.5,0.];
            for edge in 0..4 {
                let first = mesh.positions.len() as u32;
                mesh.positions.extend([corners[edge],corners[(edge+1)%4],center]);
                mesh.indices.extend([first,first+1,first+2]);
            }
        }
        let uvs = coordinates(&mesh,0..mesh.positions.len());
        assert_eq!(uvs.len(),24);
        for &(i,uv) in &uvs {
            let p = crate::scenery_finish::town_glass_coordinates(uv).unwrap();
            let expected = if i<12 { [mesh.positions[i][0],mesh.positions[i][1]] }
                else { [(mesh.positions[i][0]-4.)/2.,mesh.positions[i][1]/3.] };
            assert_eq!(p,expected,"pane-local coordinates at vertex{i}");
        }
    }
}
