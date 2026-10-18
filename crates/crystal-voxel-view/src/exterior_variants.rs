//! Cached original derivatives of the authored closed scenery meshes.
//! Positive smooth warps retain topology, material seams and rooted contacts.
use super::*;

const KINDS: [Kind; 5] = [
    Kind::CutTree,
    Kind::ShoreBoulder,
    Kind::KantoBoundaryLand,
    Kind::KantoBoundaryShore,
    Kind::KantoBoundaryPath,
];

// Normalized cross-section, taper, lean and height. Each family has three
// distinct silhouettes, rather than three copies with different rotations.
const SHRUBS: [[f32; 6]; 3] = [
    [0.95, 0.92, 0.10, -0.02, 0.02, 0.86],  // low rounded crown
    [0.84, 0.90, -0.06, 0.04, -0.02, 0.98], // upright crown
    [0.94, 0.85, 0.04, -0.04, 0.05, 0.92],  // spreading swept crown
];
const ROCKS: [[f32; 6]; 3] = [
    [0.97, 0.94, 0.12, 0.02, -0.02, 0.83], // broad weathered shoulder
    [0.88, 0.96, -0.03, -0.04, 0.02, 0.98], // upright angular stone
    [0.96, 0.88, 0.08, 0.04, 0.04, 0.91],  // asymmetric sloping stone
];

fn warp(p: Vec3, recipe: [f32; 6], stone: bool) -> Vec3 {
    let [width, depth, taper, lean_x, lean_z, height] = recipe;
    Vec3::new(
        p.x * (width - taper * p.y) + lean_x * p.y * (1. - p.x * p.x),
        p.y * height
            * if stone {
                1. + 4. * (lean_x * p.x + lean_z * p.z)
            } else {
                1.
            },
        p.z * (depth - taper * p.y * 0.6) + lean_z * p.y * (1. - p.z * p.z),
    )
}

fn generate(source: &Model, recipe: [f32; 6], stone: bool, asset: usize, faceted: bool) -> Model {
    let mut result = source.clone();
    let min = Vec3::from_array(source.min);
    let extent = Vec3::from_array(source.max) - min;
    let unit = Vec3::new(extent.x * 0.5, extent.y, extent.z * 0.5);
    let origin = min + Vec3::new(unit.x, 0., unit.z);
    for ((p, n), (&original, &normal)) in result
        .surface
        .positions
        .iter_mut()
        .zip(&mut result.surface.normals)
        .zip(source.surface.positions.iter().zip(&source.surface.normals))
    {
        let q = (Vec3::from_array(original) - origin) / unit;
        *p = (origin + warp(q, recipe, stone) * unit).to_array();
        let [width, depth, taper, lean_x, lean_z, height] = recipe;
        // Inverse-transpose of the warp Jacobian, including model units.
        let a = width - taper * q.y - 2. * lean_x * q.y * q.x;
        let b = (-taper * q.x + lean_x * (1. - q.x * q.x)) * unit.x / unit.y;
        let c = depth - taper * q.y * 0.6 - 2. * lean_z * q.y * q.z;
        let d = (-taper * q.z * 0.6 + lean_z * (1. - q.z * q.z)) * unit.z / unit.y;
        let slope_x = if stone { lean_x * 4. } else { 0. };
        let slope_z = if stone { lean_z * 4. } else { 0. };
        let jacobian = bevy::prelude::Mat3::from_cols(
            Vec3::new(a, height * q.y * slope_x * unit.y / unit.x, 0.),
            Vec3::new(b, height * (1. + slope_x * q.x + slope_z * q.z), d),
            Vec3::new(0., height * q.y * slope_z * unit.y / unit.z, c),
        );
        *n = (jacobian.inverse().transpose() * Vec3::from_array(normal))
            .normalize()
            .to_array();
    }
    if stone {
        // Tip the shoulders, then fit any raised peak proportionally rather
        // than clipping vertices into a flat cap. Ground contacts stay fixed.
        let high = result
            .surface
            .positions
            .iter()
            .map(|p| p[1])
            .fold(min.y, f32::max);
        let fit = (extent.y / (high - min.y)).min(1.);
        for (p, n) in result
            .surface
            .positions
            .iter_mut()
            .zip(&mut result.surface.normals)
        {
            p[1] = min.y + (p[1] - min.y) * fit;
            *n = Vec3::new(n[0], n[1] / fit, n[2]).normalize().to_array();
        }
        // Ordinary boulders retain their broad weathered facets. Boundary
        // posts keep the shared smooth normals across pigment seams.
        let mut uses = vec![0usize; result.surface.positions.len()];
        for &i in &result.surface.indices {
            uses[i as usize] += 1;
        }
        for ids in result.surface.indices.chunks_exact(3).filter(|_|faceted) {
            let [a, b, c] = std::array::from_fn(|i| {
                Vec3::from_array(result.surface.positions[ids[i] as usize])
            });
            let face = (b - a).cross(c - a).normalize_or_zero();
            if face.length_squared() < 0.5 {
                continue;
            }
            for &id in ids {
                if uses[id as usize] == 1 {
                    let n = &mut result.surface.normals[id as usize];
                    *n = face.lerp(Vec3::from_array(*n), 0.20).normalize().to_array();
                }
            }
        }
        let tint = [[0.80, 0.78, 0.72], [0.72, 0.77, 0.81], [0.78, 0.79, 0.70]][asset];
        for (p, color) in result
            .surface
            .positions
            .iter()
            .zip(&mut result.surface.colors)
        {
            let q = (Vec3::from_array(*p) - origin) / unit;
            let grain = 0.95 + 0.035 * (q.x * 37. + q.y * 29. + q.z * 41.).sin();
            let seam = (q.y * 18. + q.x * 5. - q.z * 9.).sin().abs();
            let mineral = if seam < 0.13 { 0.90 } else { 1.0 };
            for channel in 0..3 {
                color[channel] *= tint[channel] * grain * mineral;
            }
        }
    }
    // Retain original fitting bounds: fitting each derivative to its own
    // extrema would cancel the deliberately different crown/shoulder sizes.
    result
}

pub(crate) fn variant_model(kind: Kind, coordinates: [i32; 2]) -> &'static Model {
    let Some(family) = KINDS.iter().position(|&k| k == kind) else {
        return model(kind);
    };
    static VARIANTS: OnceLock<Vec<[Model; 3]>> = OnceLock::new();
    let models = VARIANTS.get_or_init(|| {
        KINDS
            .iter()
            .map(|&kind| {
                let recipes = if kind == Kind::CutTree { SHRUBS } else { ROCKS };
                std::array::from_fn(|asset| {
                    generate(model(kind), recipes[asset], kind != Kind::CutTree, asset, kind==Kind::ShoreBoulder)
                })
            })
            .collect()
    });
    &models[family][crate::scenery_variants::index(coordinates, family as u32 + 1)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boundary_variants_keep_continuous_normals_at_material_seams() {
        for kind in [Kind::KantoBoundaryLand,Kind::KantoBoundaryShore,Kind::KantoBoundaryPath] {
            for recipe in ROCKS {
                let m=generate(model(kind),recipe,true,0,false);
                let mut seen=std::collections::BTreeMap::new();
                for (&p,&n) in m.surface.positions.iter().zip(&m.surface.normals) {
                    if let Some(old)=seen.insert(p.map(f32::to_bits),n) {
                        assert!(Vec3::from_array(old).distance(Vec3::from_array(n))<0.0001);
                    }
                }
            }
        }
    }

    #[test]
    fn three_scenery_assets_keep_closed_topology_materials_contacts_and_footprints() {
        for kind in KINDS {
            let source = model(kind);
            let recipes = if kind == Kind::CutTree { SHRUBS } else { ROCKS };
            let variants: [Model; 3] = std::array::from_fn(|asset| {
                generate(source, recipes[asset], kind != Kind::CutTree, asset, kind==Kind::ShoreBoulder)
            });
            assert_ne!(variants[0].surface.positions, variants[1].surface.positions);
            assert_ne!(variants[1].surface.positions, variants[2].surface.positions);
            assert_ne!(variants[0].surface.positions, variants[2].surface.positions);
            for variant in variants {
                assert_eq!(variant.surface.indices, source.surface.indices);
                if kind == Kind::CutTree {
                    assert_eq!(variant.surface.colors, source.surface.colors);
                } else {
                    assert_ne!(variant.surface.colors, source.surface.colors);
                    assert!(
                        variant
                            .surface
                            .colors
                            .iter()
                            .zip(&source.surface.colors)
                            .all(|(a, b)| a[3] == b[3]
                                && (0..3).all(|c| a[c].is_finite() && a[c] >= 0. && a[c] <= b[c]))
                    );
                }
                assert_eq!(variant.surface.uvs, source.surface.uvs);
                for (p, original) in variant
                    .surface
                    .positions
                    .iter()
                    .zip(&source.surface.positions)
                {
                    assert!(p.iter().all(|v| v.is_finite()));
                    for axis in 0..3 {
                        assert!(p[axis] >= source.min[axis] - 0.001);
                        assert!(p[axis] <= source.max[axis] + 0.001);
                    }
                    if (original[1] - source.min[1]).abs() < 0.0001 {
                        assert!((p[1] - source.min[1]).abs() < 0.0001);
                    }
                }
                assert!(
                    variant
                        .surface
                        .normals
                        .iter()
                        .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 0.001)
                );
            }
        }
    }
}
