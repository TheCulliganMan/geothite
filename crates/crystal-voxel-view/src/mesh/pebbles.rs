//! Closed, irregular mineral pebbles; callers retain the original terrain backing.
use super::*;

pub(in crate::mesh) fn append(
    out: &mut SurfaceMeshData,
    seed: f32,
    palette: [[f32; 3]; 2],
    position: impl Fn(Vec3) -> Vec3,
) {
    const SIDES: usize = 7;
    let variation = |i: usize, salt: f32| {
        ((seed * 127.1 + i as f32 * 73.7 + salt).sin() * 43758.547).fract().abs()
    };
    let rings: [[Vec3; SIDES]; 3] = std::array::from_fn(|ring| {
        std::array::from_fn(|i| {
            let angle = i as f32 * std::f32::consts::TAU / SIDES as f32
                + seed * 4.1 + (variation(i, 19.) - 0.5) * 0.15;
            let radius = [0.56, 1., 0.69][ring] * (0.87 + variation(i, 31.) * 0.13);
            let y = [-0.82, -0.13, 0.56][ring]
                + (variation(i, 53. + ring as f32 * 17.) - 0.5) * 0.14;
            position(Vec3::new(angle.cos() * radius, y, angle.sin() * radius))
        })
    });
    let center = position(Vec3::ZERO);
    let mut triangle = |mut p: [Vec3; 3]| {
        let outward = (p[0] + p[1] + p[2]) / 3. - center;
        let mut n = (p[1] - p[0]).cross(p[2] - p[0]);
        if n.dot(outward) < 0. { p.swap(1, 2); n = -n; }
        if n.length_squared() < 1e-12 { return; }
        let n = n.normalize();
        let tone = 0.48 + seed * 0.22 + n.y.max(0.) * 0.12;
        let pigment = Vec3::from_array(palette[1]).lerp(Vec3::from_array(palette[0]), tone);
        let shade = 0.79 + n.dot(Vec3::new(0.4, 0.85, 0.3).normalize()).max(0.) * 0.21;
        let c = pigment * shade;
        let first = out.positions.len() as u32;
        out.positions.extend(p.map(|p| p.to_array()));
        out.normals.extend([n.to_array(); 3]);
        out.colors.extend([[c.x, c.y, c.z, 1.]; 3]);
        out.uvs.extend([crate::scenery_finish::SOURCE_PAINT_UV; 3]);
        out.indices.extend([first, first + 1, first + 2]);
    };
    for ring in 0..2 { for i in 0..SIDES {
        let j = (i + 1) % SIDES;
        triangle([rings[ring][i], rings[ring + 1][i], rings[ring + 1][j]]);
        triangle([rings[ring][i], rings[ring + 1][j], rings[ring][j]]);
    }}
    let bottom = position(Vec3::new(-0.09, -1., 0.06));
    let crown = position(Vec3::new(0.13 * (seed - 0.5), 1., -0.11));
    for i in 0..SIDES {
        let j = (i + 1) % SIDES;
        triangle([rings[0][i], bottom, rings[0][j]]);
        triangle([rings[2][i], rings[2][j], crown]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pebbles_are_closed_outward_bounded_and_have_varied_silhouettes() {
        let mut silhouettes = std::collections::BTreeSet::new();
        for seed in [0., 0.17, 0.51, 0.79, 1.] {
            let mut out = SurfaceMeshData::default();
            append(&mut out, seed, [[0.6, 0.4, 0.2], [0.2, 0.1, 0.05]], |p| p);
            let mut edges = std::collections::BTreeMap::new();
            let mut volume = 0.;
            for ids in out.indices.chunks_exact(3) {
                let p = [ids[0], ids[1], ids[2]].map(|i| Vec3::from_array(out.positions[i as usize]));
                assert!((p[1]-p[0]).cross(p[2]-p[0]).dot((p[0]+p[1]+p[2])/3.) > 0.);
                volume += p[0].dot(p[1].cross(p[2])) / 6.;
                for j in 0..3 {
                    let mut edge = [p[j].to_array().map(f32::to_bits), p[(j+1)%3].to_array().map(f32::to_bits)];
                    edge.sort(); *edges.entry(edge).or_insert(0) += 1;
                }
            }
            assert!(volume > 0. && edges.values().all(|&n| n == 2));
            assert!(out.positions.iter().flatten().all(|v| v.abs() <= 1.));
            silhouettes.insert(out.positions.iter().map(|p| p.map(f32::to_bits)).collect::<Vec<_>>());
        }
        assert_eq!(silhouettes.len(), 5);
    }
}
