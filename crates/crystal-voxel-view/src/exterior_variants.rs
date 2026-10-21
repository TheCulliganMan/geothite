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

#[derive(Clone,Copy)]
pub(crate) struct StonePose {
    pub(crate) linear:bevy::math::Mat3,
    offset:Vec2,
    pub(crate) rise:f32,
}
impl StonePose {
    /// Source-plot coordinates; offshore stones share these with the water mask.
    pub(crate) fn point(self,p:Vec2)->Vec2 {
        let q=self.linear*Vec3::new(p.x-1.,0.,p.y-1.);
        Vec2::new(q.x,q.z)+Vec2::ONE+self.offset
    }
    pub(crate) fn normal_matrix(self,tile:Vec2)->bevy::math::Mat3 {
        let units=bevy::math::Mat3::from_diagonal(Vec3::new(tile.x,1.,tile.y));
        (units*self.linear*units.inverse()).inverse().transpose()
    }
}

pub(crate) fn shore_pose(at:[i32;2])->StonePose { stone_pose(Kind::KantoBoundaryShore,at) }
pub(crate) fn land_pose(at:[i32;2])->StonePose { stone_pose(Kind::KantoBoundaryLand,at) }
fn stone_pose(kind:Kind,[x,z]:[i32;2])->StonePose {
    let land=kind==Kind::KantoBoundaryLand;
    let seed=(x as u32).wrapping_mul(0x9e3779b9)
        ^(z as u32).wrapping_mul(0x85ebca6b)^if land {0x31af_9b67}else{0xd19a_73b5};
    let unit=|salt:u32| {
        let mut h=seed^salt.wrapping_mul(0x7feb352d);
        h=(h^(h>>16)).wrapping_mul(0x846ca68b);h^=h>>16;
        (h&65535)as f32/65535.
    };
    let angle=unit(1)*std::f32::consts::TAU;
    let (s,c)=angle.sin_cos();
    let scale=Vec2::new(0.94+unit(2)*0.16,0.90+unit(3)*0.18);
    let offset=Vec2::new(unit(4)-0.5,unit(5)-0.5)*if land {0.16}else{0.26};
    let span=Vec2::splat(2.)-Vec2::from_array(KANTO_BOUNDARY_INSET)*2.;
    let m=variant_model(kind,[x,z]);
    let min=Vec2::new(m.min[0],m.min[2]);
    let extent=Vec2::new(m.max[0],m.max[2])-min;
    let rotated=m.surface.positions.iter().map(|p| {
        let q=((Vec2::new(p[0],p[2])-min)/extent-Vec2::splat(0.5))*span*scale;
        Vec2::new(c*q.x+s*q.y,-s*q.x+c*q.y).abs()
    }).fold(Vec2::ZERO,Vec2::max);
    // Fit the rotated envelope inside the same source plot. Shrink only when
    // needed; fitting every shape back to its extrema would erase variation.
    let fit=((span*0.5-offset.abs())/rotated).min_element().min(1.);
    let scale=scale*fit;
    StonePose {
        linear:bevy::math::Mat3::from_cols(Vec3::new(c*scale.x,0.,-s*scale.x),
            Vec3::Y,Vec3::new(s*scale.y,0.,c*scale.y)),
        offset,rise:if land {1.35+unit(6)*0.40}else{1.25+unit(6)*0.60},
    }
}

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

/// Cut broad weathered shoulders into the sea stones. The low slab, raked
/// crown and cleft shoulder need different volumes, not just narrow changes
/// to an egg's scale. Preserve every seated vertex so the existing optical
/// contact and native two-cell plot stay exact.
fn generate_shore(source: &Model, asset: usize) -> Model {
    let mut result = generate(source, ROCKS[asset], true, asset, false);
    let min = Vec3::from_array(source.min);
    let extent = Vec3::from_array(source.max) - min;
    let unit = Vec3::new(extent.x * 0.5, extent.y, extent.z * 0.5);
    let origin = min + Vec3::new(unit.x, 0., unit.z);
    for p in &mut result.surface.positions {
        let q = (Vec3::from_array(*p) - origin) / unit;
        if q.y <= 0.00001 { continue; }
        let shoulder = match asset {
            0 => 0.50 + q.x * 0.12 - q.z * 0.06,
            1 => 0.88 - q.x * 0.16 + q.z * 0.10,
            _ => {
                let across = (q.x * 0.82 + q.z * 0.57).abs();
                0.64 + across * 0.32 - q.z * 0.08
            }
        };
        p[1] = origin.y + q.y.min(shoulder) * unit.y;
    }
    rebuild_stone_normals(&mut result);
    result
}

/// Three raked dry-stone shoulders retain the planted foot and source topology.
/// Placement sets their actual rise independently of the shoulder cut.
fn generate_land(source:&Model,asset:usize)->Model {
    let mut result=generate(source,ROCKS[asset],true,asset,false);
    let min=Vec3::from_array(source.min);
    let extent=Vec3::from_array(source.max)-min;
    let unit=Vec3::new(extent.x*0.5,extent.y,extent.z*0.5);
    let origin=min+Vec3::new(unit.x,0.,unit.z);
    for p in &mut result.surface.positions {
        let q=(Vec3::from_array(*p)-origin)/unit;
        if q.y<=0.00001 {continue;}
        let shoulder=match asset {
            0=>0.78+q.x*0.12-q.z*0.06,
            1=>0.96-q.x*0.16+q.z*0.10,
            _=>0.84+q.x*0.10+q.z*0.07-(q.x*0.65-q.z*0.35).abs()*0.12,
        };
        p[1]=origin.y+q.y.min(shoulder)*unit.y;
    }
    rebuild_stone_normals(&mut result);
    result
}

fn rebuild_stone_normals(result:&mut Model) {
    // Rebuild welded normals from the new volume. Paint partitions share
    // their actual crease instead of exposing the old mesh's normal seams.
    let mut normals = std::collections::BTreeMap::<[u32; 3], Vec3>::new();
    for tri in result.surface.indices.chunks_exact(3) {
        let p = [tri[0], tri[1], tri[2]].map(|i|
            Vec3::from_array(result.surface.positions[i as usize]));
        let n = (p[1] - p[0]).cross(p[2] - p[0]);
        for &i in tri {
            *normals.entry(result.surface.positions[i as usize].map(f32::to_bits))
                .or_default() += n;
        }
    }
    for (p, n) in result.surface.positions.iter().zip(&mut result.surface.normals) {
        *n = normals[&p.map(f32::to_bits)].normalize().to_array();
    }
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
                    if kind == Kind::KantoBoundaryShore {
                        generate_shore(model(kind), asset)
                    } else if kind==Kind::KantoBoundaryLand {
                        generate_land(model(kind),asset)
                    } else {
                        generate(model(kind), recipes[asset], kind != Kind::CutTree, asset, kind==Kind::ShoreBoulder)
                    }
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
    fn dry_stone_rows_keep_varied_headings_heights_and_complete_native_plots() {
        let mut heights=Vec::new();let mut headings=Vec::new();
        for x in -32..32 {
            let at=[x*2,4];let pose=land_pose(at);
            heights.push(pose.rise);headings.push((pose.linear*Vec3::X).normalize());
            assert_eq!(pose.point(Vec2::ZERO),land_pose(at).point(Vec2::ZERO));
            let m=variant_model(Kind::KantoBoundaryLand,at);
            let inset=Vec2::from_array(KANTO_BOUNDARY_INSET);
            let extent=Vec2::new(m.max[0]-m.min[0],m.max[2]-m.min[2]);
            for p in &m.surface.positions {
                let local=inset+(Vec2::new(p[0]-m.min[0],p[2]-m.min[2])/extent)
                    *(Vec2::splat(2.)-inset*2.);
                let p=pose.point(local);
                assert!(p.cmpge(inset-Vec2::splat(0.00001)).all()
                    &&p.cmple(Vec2::splat(2.)-inset+Vec2::splat(0.00001)).all());
            }
        }
        assert!(heights.iter().copied().fold(0.,f32::max)
            -heights.iter().copied().fold(f32::INFINITY,f32::min)>0.30);
        assert!(headings.iter().any(|a|headings.iter().any(|b|a.dot(*b)< -0.6)));
    }

    #[test]
    fn offshore_rows_have_different_scales_offsets_and_headings_for_each_shape() {
        let mut heights:[Vec<f32>;3]=std::array::from_fn(|_|Vec::new());
        let mut centers:[Vec<Vec2>;3]=std::array::from_fn(|_|Vec::new());
        let mut headings:[Vec<Vec3>;3]=std::array::from_fn(|_|Vec::new());
        for x in 0..64 {
            let at=[x*2,0];
            let asset=crate::scenery_variants::index(at,4);
            let pose=shore_pose(at);
            heights[asset].push(pose.rise);
            centers[asset].push(pose.point(Vec2::ONE));
            headings[asset].push((pose.linear*Vec3::X).normalize());
            let repeat=shore_pose(at);
            assert_eq!(pose.point(Vec2::new(0.3,1.6)),repeat.point(Vec2::new(0.3,1.6)));
        }
        for asset in 0..3 {
            let low=heights[asset].iter().copied().fold(f32::INFINITY,f32::min);
            let high=heights[asset].iter().copied().fold(0.,f32::max);
            assert!(high>low*1.2,"each stone architecture needs unequal heights along a row");
            assert!(centers[asset].iter().any(|a|centers[asset].iter().any(|b|a.distance(*b)>0.12)),
                "repeated stones cannot retain identical centered placement");
            assert!(headings[asset].iter().any(|a|headings[asset].iter().any(|b|a.dot(*b)< -0.6)),
                "the same mineral shoulder must face different directions");
        }
    }

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
                if kind == Kind::KantoBoundaryShore { generate_shore(source, asset) }
                else if kind==Kind::KantoBoundaryLand { generate_land(source,asset) }
                else { generate(source, recipes[asset], kind != Kind::CutTree, asset, kind==Kind::ShoreBoulder) }
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

    #[test]
    fn coastal_shoulders_keep_closed_volume_seated_contacts_and_distinct_heights() {
        let source = model(Kind::KantoBoundaryShore);
        let mut heights = Vec::new();
        for asset in 0..3 {
            let old = generate(source, ROCKS[asset], true, asset, false);
            let m = generate_shore(source, asset);
            assert_eq!(m.surface.indices, old.surface.indices);
            assert_eq!(m.surface.positions.len(), old.surface.positions.len());
            assert_eq!(m.surface.colors, old.surface.colors);
            let mut welded = std::collections::BTreeMap::new();
            for ((p, before), n) in m.surface.positions.iter().zip(&old.surface.positions)
                .zip(&m.surface.normals) {
                if before[1] == source.min[1] { assert_eq!(p, before); }
                assert!(p.iter().all(|v|v.is_finite()));
                assert!((Vec3::from_array(*n).length() - 1.).abs() < 0.0001);
                if let Some(prior) = welded.insert(p.map(f32::to_bits), *n) {
                    assert_eq!(prior, *n, "paint seams share the sculpted normal");
                }
            }
            let mut edges = std::collections::BTreeMap::new();
            let mut volume = 0.;
            for face in m.surface.indices.chunks_exact(3) {
                let p = [face[0],face[1],face[2]].map(|i|
                    Vec3::from_array(m.surface.positions[i as usize]));
                let cross = (p[1]-p[0]).cross(p[2]-p[0]);
                assert!(cross.length_squared() > 0.00000001);
                let normal = face.iter().map(|&i|
                    Vec3::from_array(m.surface.normals[i as usize])).sum::<Vec3>();
                assert!(cross.dot(normal) > 0., "outward volume in asset {asset}");
                volume += p[0].dot(p[1].cross(p[2])) / 6.;
                for [a,b] in [[p[0],p[1]],[p[1],p[2]],[p[2],p[0]]] {
                    let key = |p: Vec3| p.to_array().map(|v|(v*1_000_000.).round() as i32);
                    let (a,b) = (key(a),key(b));
                    let e = edges.entry((a.min(b),a.max(b))).or_insert((0,0));
                    e.0 += 1; e.1 += if a < b {1}else{-1};
                }
            }
            assert!(volume > 0.5);
            assert!(edges.values().all(|e|*e==(2,0)), "closed water-seated shell");
            heights.push(m.surface.positions.iter().map(|p|p[1]).fold(0.,f32::max));
        }
        assert!(heights[1] > heights[0] * 1.45, "low slabs and taller shoulders must both recur");
    }
}
