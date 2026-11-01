//! Three authored CC0 broadleaf models, kept upright and rooted in native plots.
use super::*;
use std::sync::OnceLock;
mod branches;

#[derive(serde::Deserialize)]
struct Model {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
    foliage_start: usize,
    #[serde(skip)]
    living_wood: Vec<u32>,
    #[serde(skip)]
    source_height: f32,
    #[serde(skip)]
    shoot_sites: Vec<usize>,
}
fn models() -> &'static [Model; 3] {
    static MODELS: OnceLock<[Model; 3]> = OnceLock::new();
    MODELS.get_or_init(|| {
        [
            crate::model_storage::include_model!("models/deciduous/spreading.mesh.json"),
            crate::model_storage::include_model!("models/deciduous/forked.mesh.json"),
            crate::model_storage::include_model!("models/deciduous/upright.mesh.json"),
        ]
        .map(|source| {
            let mut model: Model = crate::model_storage::parse(source)
                .expect("validated authored deciduous model");
            model.source_height = model.positions.iter().map(|p| p[1]).fold(0., f32::max);
            let living = branches::living_geometry(&model);
            model.living_wood = living.indices;
            model.shoot_sites = living.shoot_sites;
            model
        })
    })
}
pub(super) fn height(tile: f32, coordinates: [i32; 2]) -> f32 {
    let v = variation(coordinates[0], coordinates[1]);
    // A broad age range puts young crowns beneath the mature leaders. Keep
    // the original models and planted footprints throughout that range.
    let age = non_ancient_height_factor(coordinates) - 0.5;
    tile * (2.8 + age.powf(1.4) * 8.0)
        * [0.94, 1.0, 1.12][v.index]
}
fn width(coordinates: [i32; 2]) -> f32 {
    0.74 + (variation(coordinates[0], coordinates[1]).width - 0.84) * 1.75
}
fn crown_spread(coordinates: [i32; 2]) -> f32 {
    // Mature branch systems spread as they grow; height alone must not turn
    // a spreading tree into a narrow pole. The rooted flare keeps its width.
    1.0 + (height(1., coordinates) - 4.).max(0.) * 0.20
}
pub(super) fn turn(coordinates: [i32;2]) -> f32 {
    let [x,z]=coordinates;
    let mut h=(x as u32).wrapping_mul(0x85eb_ca6b)
        ^(z as u32).wrapping_mul(0xc2b2_ae35)^0x6a09_e667;
    h=(h^(h>>16)).wrapping_mul(0x7feb_352d);h^=h>>15;
    (h&65535) as f32/65535.*std::f32::consts::TAU
}
pub(super) fn canopy_margin(coordinates: [i32;2]) -> f32 {0.75*crown_spread(coordinates)}
pub(super) fn width_ratio(coordinates: [i32; 2], prototype: [i32; 2]) -> f32 {
    width(coordinates) * crown_spread(coordinates)
        / (width(prototype) * crown_spread(prototype))
}

struct CrownShape {
    source_height: f32,
    source_floor: f32,
    crown_floor: f32,
    height: f32,
    stem_width: f32,
    spread: f32,
    leaf_scale: f32,
}
impl CrownShape {
    fn new(model: &Model, spacing: f32, tile: f32, coordinates: [i32; 2]) -> Self {
        let source_height = model.source_height;
        let source_floor = model.positions[model.foliage_start..].iter()
            .map(|p| p[1] / source_height).fold(1., f32::min);
        let height=height(tile,coordinates);
        // Lower foliage onto mature forks without dropping young crowns
        // into the adjacent path's eye-height clearance.
        let maturity=((height/tile-4.)/4.).clamp(0.,1.);
        Self { source_height, source_floor,
            crown_floor: source_floor+(source_floor.min(0.36)-source_floor)*maturity,
            height, stem_width: spacing * 0.86 * width(coordinates),
            spread: crown_spread(coordinates),
            leaf_scale: spacing * 0.86 * width(coordinates) * crown_spread(coordinates).min(1.35) }
    }
    fn vertical(&self, t: f32) -> (f32, f32) {
        let slope = if t < self.source_floor {
            self.crown_floor / self.source_floor
        } else { (1. - self.crown_floor) / (1. - self.source_floor) };
        let y = if t < self.source_floor { t * slope }
            else { self.crown_floor + (t - self.source_floor) * slope };
        (y * self.height, slope * self.height / self.source_height)
    }
    fn radial(&self, t: f32) -> (f32, f32) {
        let u = ((t - 0.08) / 0.20).clamp(0., 1.);
        let spread = 1. + (self.spread - 1.) * u * u * (3. - 2. * u);
        let derivative = (self.spread - 1.) * 6. * u * (1. - u)
            / (0.20 * self.source_height);
        (self.stem_width * spread, self.stem_width * derivative)
    }
    fn point(&self, p: Vec3) -> Vec3 {
        let t = p.y / self.source_height;
        let (y, _) = self.vertical(t);
        let (r, _) = self.radial(t);
        Vec3::new(p.x * r, y, p.z * r)
    }
    fn normal(&self, p: Vec3, n: Vec3) -> Vec3 {
        let t = p.y / self.source_height;
        let (_, dy) = self.vertical(t);
        let (r, dr) = self.radial(t);
        // Inverse transpose of the actual flared branch deformation.
        Vec3::new(n.x / r, (n.y - dr * (p.x * n.x + p.z * n.z) / r) / dy,
            n.z / r).normalize()
    }
}

// Fine compound sprays occupy the original authored sites. Leaves stop growing
// with a mature crown's spread, so a close bough does not become giant shingles.
// Every leaflet is a closed, cupped six-edge volume, with its own roll.
#[derive(Clone, Copy)]
struct LeafSpray {
    center: Vec3,
    normal: Vec3,
    along: Vec3,
    across: Vec3,
    radius: f32,
    breadth: f32,
    color: [f32;4],
}
fn leaf_site(model: &Model, start: usize) -> LeafSpray {
    let source = &model.positions[start..start + 12];
    let center = (Vec3::from_array(source[10]) + Vec3::from_array(source[11])) * 0.5;
    let normal = (Vec3::from_array(source[10]) - Vec3::from_array(source[11])).normalize();
    let axis = (0..5).map(|i| Vec3::from_array(source[i]) - Vec3::from_array(source[i + 5]))
        .max_by(|a,b| a.length_squared().total_cmp(&b.length_squared())).unwrap();
    let along = axis.normalize();
    let across = normal.cross(along).normalize();
    let radius = axis.length() * 0.5;
    let breadth = source[..10].iter().map(|&p| (Vec3::from_array(p) - center).dot(across).abs())
        .fold(0., f32::max);
    let color = std::array::from_fn(|i| (start..start + 12).map(|j| model.colors[j][i]).sum::<f32>() / 12.);
    LeafSpray { center, normal, along, across, radius, breadth, color }
}
fn append_leaf_spray(mesh: &mut SurfaceMeshData, model: &Model, start: usize, shape: &CrownShape) {
    let mut site = leaf_site(model,start);
    site.center = shape.point(site.center);
    append_leaf_fan(mesh,&site,shape.leaf_scale);
}
fn append_leaf_fan(mesh: &mut SurfaceMeshData, site: &LeafSpray, scale: f32) {
    let LeafSpray {center,normal,along,across,radius,breadth,color} = *site;
    for leaf in 0..12 {
        let pair=(leaf/2) as f32;
        let side=if leaf%2==0 {-1.}else{1.};
        let a=-0.64+pair*0.256;
        let b=side*(0.34+0.06*(pair*1.7).sin());
        let angle=side*(0.38+0.07*(pair*1.3).cos());
        let roll=side*(0.30+pair*0.04);
        let turn = Quat::from_axis_angle(normal,angle)*Quat::from_axis_angle(along,roll);
        let origin = center
            + (along * radius * a + across * breadth * b) * scale;
        append_blade(mesh, origin, turn * along, turn * across, turn * normal,
            radius * scale * 0.30, breadth * scale * 0.35, color);
    }
}

fn append_shoots(mesh: &mut SurfaceMeshData, model: &Model, shape: &CrownShape, tile: f32) -> Vec<LeafSpray> {
    let maturity = ((shape.height / tile - 4.) / 4.).clamp(0.,1.);
    if maturity < 0.05 { return Vec::new(); }
    let foliage = (model.foliage_start..model.positions.len()).step_by(24).map(|start| {
        let site = leaf_site(model,start);
        (shape.point(site.center),(site.radius*shape.leaf_scale).powi(2))
    }).collect::<Vec<_>>();
    let candidates = model.shoot_sites.iter().copied().filter(|&vertex| {
        let point = shape.point(Vec3::from_array(model.positions[vertex]));
        foliage.iter().all(|&(center,radius2)|point.distance_squared(center)>radius2)
    }).collect::<Vec<_>>();
    let count = candidates.len().min(foliage.len()/3);
    (0..count).map(|index| {
        let vertex = candidates[index*candidates.len()/count];
        let source = Vec3::from_array(model.positions[vertex]);
        let normal = shape.normal(source,Vec3::from_array(model.normals[vertex]));
        let direction = Vec3::new(normal.x,normal.y.clamp(-0.1,0.4)+0.35,normal.z).normalize();
        let mut site = leaf_site(model,model.foliage_start
            + ((index * 37 + 11) % ((model.positions.len()-model.foliage_start)/12))*12);
        let scale = shape.leaf_scale * (0.65 + maturity * 0.35);
        site.radius *= scale;
        site.breadth *= scale;
        let root = shape.point(source) - normal * (tile * 0.006);
        let tip = root + direction * (site.radius * 1.25);
        append_shoot_stem(mesh,root,tip,tile*0.006,model.colors[vertex]);
        site.center = root + direction * (site.radius * 0.65);
        site.along = direction;
        let reference = if direction.y.abs()>0.99 {Vec3::X}else{Vec3::Y};
        let roll = ((index as f32*0.618034).fract()*2.-1.)*1.15;
        site.across = Quat::from_axis_angle(direction,roll)*direction.cross(reference).normalize();
        site.normal = site.along.cross(site.across).normalize();
        site
    }).collect()
}

fn append_shoot_stem(mesh: &mut SurfaceMeshData, root: Vec3, tip: Vec3, radius: f32, color: [f32;4]) {
    let first = mesh.positions.len() as u32;
    let axis = (tip-root).normalize();
    let reference = if axis.y.abs()>0.99 {Vec3::X}else{Vec3::Y};
    let side = axis.cross(reference).normalize();
    let up = side.cross(axis).normalize();
    for row in 0..3 {
        let t = row as f32 * 0.5;
        let center = root.lerp(tip,t) + Vec3::Y * radius * 3. * (std::f32::consts::PI*t).sin();
        for ring in 0..6 {
            let angle = ring as f32 * std::f32::consts::TAU / 6.;
            let radial = side*angle.cos()+up*angle.sin();
            mesh.positions.push((center+radial*radius*(1.-0.85*t)).to_array());
        }
    }
    mesh.positions.extend([root.to_array(),tip.to_array()]);
    let mut normals = [Vec3::ZERO;20];
    let mut triangle = |ids: [u32;3]| {
        let p = ids.map(|i|Vec3::from_array(mesh.positions[(first+i) as usize]));
        let face = (p[1]-p[0]).cross(p[2]-p[0]);
        for i in ids { normals[i as usize] += face; }
        mesh.indices.extend(ids.map(|i|first+i));
    };
    for ring in 0..6 {
        let next = (ring+1)%6;
        for row in 0..2 {
            let a = row*6+ring; let b = (row+1)*6+ring;
            triangle([a,b,row*6+next]);
            triangle([row*6+next,b,(row+1)*6+next]);
        }
        triangle([18,ring,next]);
        triangle([19,12+next,12+ring]);
    }
    mesh.normals.extend(normals.map(|n|n.normalize().to_array()));
    mesh.colors.extend([color;20]);
    mesh.uvs.extend([[0.;2];20]);
}

fn append_blade(mesh: &mut SurfaceMeshData, center: Vec3, along: Vec3, across: Vec3,
    normal: Vec3, length: f32, width: f32, color: [f32; 4]) {
    let first = mesh.positions.len() as u32;
    for [a,b] in [[1.,0.],[0.40,0.80],[-0.45,1.],[-1.,0.],[-0.45,-1.],[0.40,-0.80]] {
        mesh.positions.push((center + along * (a * length) + across * (b * width)).to_array());
    }
    mesh.positions.push((center + normal * (length * 0.11)).to_array());
    mesh.positions.push((center - normal * (length * 0.035)).to_array());
    let mut normals = [Vec3::ZERO; 8];
    for edge in 0..6u32 {
        for triangle in [[6,edge,(edge+1)%6],[7,(edge+1)%6,edge]] {
            let p = triangle.map(|i| Vec3::from_array(mesh.positions[(first+i) as usize]));
            let face = (p[1]-p[0]).cross(p[2]-p[0]);
            for i in triangle { normals[i as usize] += face; }
            mesh.indices.extend(triangle.map(|i| first+i));
        }
    }
    mesh.normals.extend(normals.map(|n| n.normalize().to_array()));
    mesh.colors.extend([color;8]);
    mesh.uvs.extend([[0.;2];8]);
}
pub(super) fn append_canopy(
    mesh: &mut TerrainMeshData,
    bounds: [f32; 4],
    base: f32,
    tile: f32,
    coordinates: [i32; 2],
    _neighbors: [bool; 4],
) {
    let v = variation(coordinates[0], coordinates[1]);
    let model = &models()[v.index];
    let spacing = (bounds[1] - bounds[0]).min(bounds[3] - bounds[2]);
    let offset = grove_trunk_offset(coordinates, spacing);
    let root = Vec3::new(
        (bounds[0] + bounds[1]) * 0.5 + offset.x,
        base,
        (bounds[2] + bounds[3]) * 0.5 + offset.y,
    );
    let shape = CrownShape::new(model, spacing, tile, coordinates);
    let mut local = SurfaceMeshData::default();
    let mut wood_remap = vec![u32::MAX; model.foliage_start];
    for &i in &model.living_wood { wood_remap[i as usize] = 0; }
    for i in 0..model.foliage_start {
        if wood_remap[i] == u32::MAX { continue; }
        wood_remap[i] = local.positions.len() as u32;
        let p = Vec3::from_array(model.positions[i]);
        local.positions.push(shape.point(p).to_array());
        local.normals.push(shape.normal(p, Vec3::from_array(model.normals[i])).to_array());
        local.colors.push(model.colors[i]);
        local.uvs.push([0.;2]);
    }
    local.indices.extend(model.living_wood.iter().map(|&i| wood_remap[i as usize]));
    let shoots = append_shoots(&mut local,model,&shape,tile);
    let foliage_start = local.positions.len();
    // Alternating authored sites keep the branch systems populated while
    // the finer compound sprays avoid multiplying every overlapping clump.
    for start in (model.foliage_start..model.positions.len()).step_by(24) {
        append_leaf_spray(&mut local, model, start, &shape);
    }
    for shoot in &shoots { append_leaf_fan(&mut local,shoot,1.); }
    let top = local.positions.iter().map(|p| p[1]).fold(0., f32::max);
    let vertical_fit = shape.height / top;
    let rotation = Quat::from_rotation_y(turn(coordinates));
    let mut fit = 1f32;
    // Authoring roots are genuinely modeled. Fit their complete flare, rather
    // than stretching the source plot or clipping roots onto neighboring paths.
    for &p in &local.positions[..foliage_start] {
        if p[1] * vertical_fit > tile * 0.18 {
            continue;
        }
        let d = rotation * Vec3::from_array(p);
        for (delta, center, low, high) in [
            (d.x, root.x, bounds[0], bounds[1]),
            (d.z, root.z, bounds[2], bounds[3]),
        ] {
            if delta > 0. {
                fit = fit.min((high - center) / delta);
            }
            if delta < 0. {
                fit = fit.min((low - center) / delta);
            }
        }
    }
    let scale = Vec3::new(fit, vertical_fit, fit);
    let normals = Mat3::from_quat(rotation) * Mat3::from_diagonal(scale.recip());
    let first = mesh.solid.positions.len() as u32;
    for (i, (&p, &n)) in local.positions.iter().zip(&local.normals).enumerate() {
        let point = root + rotation * (Vec3::from_array(p) * scale);
        let mut color = local.colors[i];
        if i >= foliage_start {
            let pigment = leaf_pigment(point, tile * 1.5);
            // Keep the three models in the same leafy woodland palette while
            // retaining the artist's baked shade variation on individual leaves.
            color = [
                (color[0] * 0.72 + color[1] * 0.22) * pigment[0] * v.tint[0],
                color[1] * 0.88 * pigment[1] * v.tint[1],
                (color[2] + color[1] * 0.28) * pigment[2] * v.tint[2],
                1.,
            ];
        }
        mesh.solid.positions.push(point.to_array());
        mesh.solid
            .normals
            .push((normals * Vec3::from_array(n)).normalize().to_array());
        mesh.solid.colors.push(color);
        mesh.solid.uvs.push([0.; 2]);
    }
    mesh.solid
        .indices
        .extend(local.indices.iter().map(|&i| first + i));
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn route_broadleaf_leaders_and_medium_crowns_keep_distinct_instance_scales() {
        let mut low=f32::INFINITY;
        let mut high=0.0_f32;
        let mut narrow=f32::INFINITY;
        let mut broad=0.0_f32;
        for z in -16..16 {for x in -16..16 {
            let at=[x,z];
            low=low.min(height(1.,at));high=high.max(height(1.,at));
            narrow=narrow.min(width(at));broad=broad.max(width(at));
            let prototype=[4,8];
            let ratio=instance_vertical_ratio(GroveFamily::KantoDeciduous,at,prototype,[false;4]);
            assert!((height(1.,prototype)*ratio-height(1.,at)).abs()<0.00001,
                "cached trees must retain the placement's complete height range");
            assert!((width(prototype)*crown_spread(prototype)*width_ratio(at,prototype)
                -width(at)*crown_spread(at)).abs()<0.00001,
                "shared instances must retain each placement's mature branch spread");
        }}
        assert!(high>low*4.0,"route woods need young crowns beneath their taller leaders");
        assert!(broad>narrow*1.3,"crown widths must break up the repeated silhouette");
        assert!(low>2.6 && high<12.2,"young and mature trees must retain their intended scales");
    }
    #[test]
    fn three_authored_models_have_distinct_branches_and_solid_cupped_leaves() {
        let models = models();
        assert_ne!(models[0].positions.len(), models[1].positions.len());
        assert_ne!(models[1].positions.len(), models[2].positions.len());
        for m in models {
            assert!(m.foliage_start > 4000 && m.foliage_start < m.positions.len());
            assert!(m.positions.len() < 20_000);
            assert!(
                m.colors[m.foliage_start..].iter().all(|c| c[1] > 0.05),
                "source card borders must not turn modeled leaves black"
            );
            assert_eq!((m.positions.len() - m.foliage_start) % 12, 0);
            assert!(
                m.positions
                    .iter()
                    .all(|p| p[1] >= 0. && p.iter().all(|v| v.is_finite()))
            );
            for ids in m
                .indices
                .chunks_exact(3)
                .filter(|ids| ids.iter().all(|&i| i as usize >= m.foliage_start))
            {
                let p: [Vec3; 3] =
                    std::array::from_fn(|j| Vec3::from_array(m.positions[ids[j] as usize]));
                let face = (p[1] - p[0]).cross(p[2] - p[0]);
                let n = ids
                    .iter()
                    .map(|&i| Vec3::from_array(m.normals[i as usize]))
                    .sum::<Vec3>();
                assert!(face.length_squared() > 1e-14 && face.dot(n) > 0.);
            }
        }
    }
    #[test]
    fn finer_leaf_sprays_keep_coverage_closed_faces_and_lighting_from_every_side() {
        for model in models() {
            let shape=CrownShape::new(model,32.,16.,[4,8]);
            for start in (model.foliage_start..model.positions.len()).step_by(12) {
                let mut mesh=SurfaceMeshData::default();
                append_leaf_spray(&mut mesh,model,start,&shape);
                assert_eq!(mesh.positions.len(),96);
                assert_eq!(mesh.indices.len(),432);
                let source=&model.positions[start..start+12];
                let center=(Vec3::from_array(source[10])+Vec3::from_array(source[11]))*0.5;
                let normal=(Vec3::from_array(source[10])-Vec3::from_array(source[11])).normalize();
                let area=(0..10).map(|i| (Vec3::from_array(source[i])-center)
                    .cross(Vec3::from_array(source[(i+1)%10])-center).dot(normal)).sum::<f32>().abs()*0.5
                    *shape.leaf_scale.powi(2);
                let paired_area=mesh.indices.chunks_exact(3).filter(|ids|ids[0]%8==6).map(|ids| {
                    let p=ids.iter().map(|&i|Vec3::from_array(mesh.positions[i as usize])).collect::<Vec<_>>();
                    (p[1]-p[0]).cross(p[2]-p[0]).dot(normal).abs()*0.5
                }).sum::<f32>();
                assert!((0.98..1.02).contains(&(paired_area/area)),
                    "compound sprays retain coverage at their bounded physical leaf scale");
                let faces=(0..12).map(|blade| {
                    (Vec3::from_array(mesh.positions[blade*8+6])
                        -Vec3::from_array(mesh.positions[blade*8+7])).normalize()
                }).collect::<Vec<_>>();
                assert!(faces.iter().any(|a|faces.iter().any(|b|a.dot(*b)<0.88)),
                    "a leaf fan must have depth when viewed along its original plane");
                for blade in (0..96).step_by(8) {
                    let mut edges=std::collections::BTreeMap::new();
                    for ids in mesh.indices.chunks_exact(3).filter(|ids|ids[0]as usize>=blade
                        &&(ids[0]as usize)<blade+8) {
                        let p=ids.iter().map(|&i|Vec3::from_array(mesh.positions[i as usize])).collect::<Vec<_>>();
                        let face=(p[1]-p[0]).cross(p[2]-p[0]);
                        let n=ids.iter().map(|&i|Vec3::from_array(mesh.normals[i as usize])).sum::<Vec3>();
                        assert!(face.length_squared()>1e-12 && face.dot(n)>0.);
                        for edge in [[ids[0],ids[1]],[ids[1],ids[2]],[ids[2],ids[0]]] {
                            *edges.entry((edge[0].min(edge[1]),edge[0].max(edge[1]))).or_insert(0)+=1;
                        }
                    }
                    assert!(edges.values().all(|&n|n==2),"every blade must be a closed volume");
                }
                assert!(mesh.normals.iter().all(|n|(Vec3::from_array(*n).length()-1.).abs()<0.001));
            }
        }
    }
    #[test]
    fn leafy_shoots_are_bounded_closed_growth_grafted_to_real_living_boughs() {
        let mut upright_shoots = 0;
        for (index,model) in models().iter().enumerate() {
            let at = (0..64).flat_map(|z|(0..64).map(move |x|[x,z]))
                .find(|at|variation(at[0],at[1]).index==index && height(1.,*at)>9.).unwrap();
            let shape = CrownShape::new(model,32.,16.,at);
            let mut stems = SurfaceMeshData::default();
            let shoots = append_shoots(&mut stems,model,&shape,16.);
            if index==2 { upright_shoots = shoots.len(); }
            assert!(shoots.len() <= (model.positions.len()-model.foliage_start)/24/3);
            assert_eq!(stems.positions.len(),shoots.len()*20);
            assert_eq!(stems.indices.len(),shoots.len()*108);
            for stem in 0..shoots.len() {
                let first = stem*20;
                let root = Vec3::from_array(stems.positions[first+18]);
                assert!(model.shoot_sites.iter().any(|&i| {
                    let p = Vec3::from_array(model.positions[i]);
                    root.distance(shape.point(p)) <= 16.*0.0061
                }),"every shoot must be grafted directly to an authored living bough");
                let mut edges = std::collections::BTreeMap::new();
                let mut volume = 0.;
                for ids in stems.indices[stem*108..(stem+1)*108].chunks_exact(3) {
                    let p = ids.iter().map(|&i|Vec3::from_array(stems.positions[i as usize])).collect::<Vec<_>>();
                    let face = (p[1]-p[0]).cross(p[2]-p[0]);
                    let normal = ids.iter().map(|&i|Vec3::from_array(stems.normals[i as usize])).sum::<Vec3>();
                    assert!(face.length_squared()>1e-10 && face.dot(normal)>0.);
                    volume += (p[0]-root).dot((p[1]-root).cross(p[2]-root))/6.;
                    for edge in [[ids[0],ids[1]],[ids[1],ids[2]],[ids[2],ids[0]]] {
                        *edges.entry((edge[0].min(edge[1]),edge[0].max(edge[1]))).or_insert(0)+=1;
                    }
                }
                assert!(volume>0. && edges.values().all(|&count|count==2),
                    "new twigs must have complete outward sides and both closed ends");
            }
            let young = (0..64).flat_map(|z|(0..64).map(move |x|[x,z]))
                .find(|at|variation(at[0],at[1]).index==index && height(1.,*at)<4.).unwrap();
            assert!(append_shoots(&mut SurfaceMeshData::default(),model,
                &CrownShape::new(model,32.,16.,young),16.).is_empty(),
                "young trees must keep their existing low crown clearance");
        }
        assert!(upright_shoots>20,"bare upright boughs need real leafy growth along their length");
    }
    #[test]
    fn flared_branch_normals_follow_the_actual_deformed_surface() {
        for model in models() {
            let shape=CrownShape::new(model,32.,16.,[4,8]);
            let n=Vec3::new(0.3,0.6,0.7).normalize();
            let u=n.cross(Vec3::X).normalize();let v=n.cross(u);
            for t in [0.02,0.12,0.18,0.30,0.5,0.8] {
                let p=Vec3::new(0.22,t*shape.source_height,-0.17);
                let du=shape.point(p+u*0.001)-shape.point(p-u*0.001);
                let dv=shape.point(p+v*0.001)-shape.point(p-v*0.001);
                assert!(du.cross(dv).normalize().dot(shape.normal(p,n))>0.999,
                    "branch lighting must follow its actual flared surface");
            }
        }
    }
    #[test]
    fn deciduous_trees_stay_upright_rooted_and_deterministic_at_native_scales() {
        for tile in [8., 16.] {
            for coordinates in [[0, 0], [4, 8], [-12, 2], [29, 4], [7, 11], [1, 8]] {
                let mut a = TerrainMeshData::default();
                let mut b = TerrainMeshData::default();
                let bounds = [0., tile * 2., 0., tile * 2.];
                append_canopy(&mut a, bounds, 3., tile, coordinates, [false; 4]);
                append_canopy(&mut b, bounds, 3., tile, coordinates, [true; 4]);
                assert_eq!(a.solid.positions, b.solid.positions);
                let bottom = a
                    .solid
                    .positions
                    .iter()
                    .map(|p| p[1])
                    .fold(f32::INFINITY, f32::min);
                let top = a.solid.positions.iter().map(|p| p[1]).fold(0., f32::max);
                assert!((bottom - 3.).abs() < 0.001);
                assert!((top - 3. - height(tile, coordinates)).abs() < 0.001);
                assert!(top > 3. + tile * 2.6 && top < 3. + tile * 12.2);
                assert!(
                    a.solid
                        .positions
                        .iter()
                        .filter(|p| p[1] < 3. + tile * 0.18)
                        .all(|p| p[0] >= -0.001
                            && p[0] <= tile * 2. + 0.001
                            && p[2] >= -0.001
                            && p[2] <= tile * 2. + 0.001)
                );
                assert!(
                    a.solid
                        .normals
                        .iter()
                        .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 0.001)
                );
            }
        }
    }
}
