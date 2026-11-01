//! Stable per-placement variation. Camera movement never changes a tree.
use crate::{
    mesh::{SurfaceMeshData, TerrainMeshData},
};
use bevy::prelude::*;

#[path = "foliage/park_tree.rs"]
mod park_tree;
mod pine;
mod ancient;
mod deciduous;
mod winter_pine;
pub(crate) use park_tree::append_park_tree;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GroveFamily { JohtoPine, AncientForest, KantoDeciduous, ParkBroadleaf, WinterPine }
impl GroveFamily {
    pub(crate) fn from_drawing(tileset: &str, width: usize, height: usize) -> Self {
        // Forest's small 2x2 Headbutt drawing and 4x4 ancient canopy are
        // different authored objects. Map identity must never conflate them.
        if tileset == "forest" && width == 2 && height == 2 {
            Self::JohtoPine
        } else { Self::from_tileset(tileset) }
    }
    pub(crate) fn from_tileset(tileset: &str) -> Self {
        match tileset {"forest"=>Self::AncientForest,"kanto"=>Self::KantoDeciduous,"park"=>Self::ParkBroadleaf,_=>Self::JohtoPine}
    }
    pub(crate) fn label(self) -> &'static str {
        match self { Self::JohtoPine => "johto/tree", Self::AncientForest => "forest/ancient_tree", Self::KantoDeciduous => "world-exterior/kanto_canopy", Self::ParkBroadleaf => "world-exterior/park_tree", Self::WinterPine => "winter/pine" }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Variation {
    index: usize,
    height: f32,
    width: f32,
    turn: f32,
    tint: [f32; 3],
    offset: [f32; 2],
}
fn variation(x: i32, z: i32) -> Variation {
    let mut value = (x as u32).wrapping_mul(0x9e3779b9) ^ (z as u32).wrapping_mul(0x85ebca6b);
    value ^= value >> 16;
    value = value.wrapping_mul(0x7feb352d);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846ca68b);
    value ^= value >> 16;
    let unit = |shift: u32| ((value >> shift) & 255) as f32 / 255.;
    Variation {
        index: crate::scenery_variants::index([x, z], 0),
        height: 0.90 + unit(8) * 0.70,
        width: 0.84 + unit(16) * 0.16,
        turn: ((value >> 24) & 3) as f32 * std::f32::consts::FRAC_PI_2,
        offset: [unit(0) * 2.0 - 1.0, unit(24) * 2.0 - 1.0],
        tint: [
            0.94 + unit(0) * 0.08,
            0.96 + unit(8) * 0.06,
            0.94 + unit(16) * 0.09,
        ],
    }
}

/// Stable pose inputs for the runtime forest prototype library.
pub(crate) fn instance_variation(coordinates: [i32; 2]) -> (usize, f32, f32) {
    let v = variation(coordinates[0], coordinates[1]);
    (v.index, v.height, v.turn)
}

/// Per-tree height, stable across scrolling and rebuilds. This factor is only
/// used by ordinary trees; ancient trees and formal park trees keep their scale.
pub(crate) fn non_ancient_height_factor([x,z]: [i32;2]) -> f32 {
    let mut n = (x as u32).wrapping_mul(0x9e3779b9)
        ^ (z as u32).wrapping_mul(0x85ebca6b) ^ 0x41c64e6d;
    n ^= n >> 16;
    n = n.wrapping_mul(0x7feb352d);
    n ^= n >> 15;
    n = n.wrapping_mul(0x846ca68b);
    n ^= n >> 16;
    0.5 + (n & 65535) as f32 / 65535.
}

pub(crate) fn instance_vertical_ratio(family: GroveFamily, coordinates: [i32;2],
    prototype: [i32;2], neighbors: [bool;4]) -> f32 {
    match family {
        GroveFamily::JohtoPine | GroveFamily::WinterPine => pine::stem_height(1.,coordinates,neighbors)
            / pine::stem_height(1.,prototype,neighbors),
        GroveFamily::AncientForest => variation(coordinates[0],coordinates[1]).height
            / variation(prototype[0],prototype[1]).height,
        GroveFamily::KantoDeciduous => deciduous::height(1.,coordinates)/deciduous::height(1.,prototype),
        GroveFamily::ParkBroadleaf => park_tree::dimensions(1.,1.,coordinates).0
            / park_tree::dimensions(1.,1.,prototype).0,
    }
}

/// Build the cheaper pine render copy; other families retain full detail.
pub(crate) fn append_distant_pine(
    mesh: &mut TerrainMeshData,
    bounds: [f32; 4],
    base: f32,
    tile: f32,
    coordinates: [i32; 2],
    neighbors: [bool; 4],
) {
    pine::append_distant(mesh, bounds, base, tile, coordinates, neighbors);
    mesh.solid
        .cutaway_ranges
        .push(0..mesh.solid.positions.len());
}

/// Ordinary source-authenticated Johto groves use branching pines.
pub(crate) fn append_grove_tree(mesh: &mut TerrainMeshData, bounds: [f32;4], base: f32,
    tile: f32, coordinates: [i32;2], neighbors: [bool;4]) {
    append_family_grove(mesh,bounds,base,tile,coordinates,neighbors,GroveFamily::JohtoPine);
}

pub(crate) fn append_family_grove(mesh: &mut TerrainMeshData, bounds: [f32;4], base: f32,
    tile: f32, coordinates: [i32;2], neighbors: [bool;4], family: GroveFamily) {
    let _trace = crate::loading_trace::span("tree.generate", || serde_json::json!({
        "coordinates": coordinates, "neighbors": neighbors, "family": family.label()}));
    match family {
        GroveFamily::JohtoPine => pine::append(mesh,bounds,base,tile,coordinates,neighbors),
        GroveFamily::WinterPine => {
            let first = mesh.solid.positions.len();
            winter_pine::append(mesh,bounds,base,tile,coordinates,neighbors);
            mesh.solid.cutaway_ranges.push(first..mesh.solid.positions.len());
        },
        GroveFamily::AncientForest => ancient::append(mesh,bounds,base,tile,coordinates,neighbors),
        GroveFamily::ParkBroadleaf => park_tree::append_park_tree(mesh,bounds,base,tile,coordinates),
        GroveFamily::KantoDeciduous => {
            let first=mesh.solid.positions.len();
            deciduous::append_canopy(mesh,bounds,base,tile,coordinates,neighbors);
            mesh.solid.cutaway_ranges.push(first..mesh.solid.positions.len());
        },
    }
}

/// Canopies are fitted and shifted independently of the ground layer at runtime.
pub(crate) fn append_family_canopy(mesh: &mut TerrainMeshData, bounds: [f32;4], base: f32,
    tile: f32, coordinates: [i32;2], neighbors: [bool;4], family: GroveFamily) {
    let first = mesh.solid.positions.len();
    match family {
        GroveFamily::JohtoPine => pine::append_canopy(mesh,bounds,base,tile,coordinates,neighbors),
        GroveFamily::WinterPine => winter_pine::append(mesh,bounds,base,tile,coordinates,neighbors),
        GroveFamily::AncientForest => ancient::append_canopy(mesh,bounds,base,tile,coordinates,neighbors),
        GroveFamily::KantoDeciduous => deciduous::append_canopy(mesh,bounds,base,tile,coordinates,neighbors),
        GroveFamily::ParkBroadleaf => {
            park_tree::append_park_tree(mesh,bounds,base,tile,coordinates);
            return;
        },
    }
    if mesh.solid.positions.len() > first {
        mesh.solid.cutaway_ranges.push(first..mesh.solid.positions.len());
    }
}



pub(crate) fn instance_width_ratio(family:GroveFamily,coordinates:[i32;2],prototype:[i32;2])->f32 {
    if family==GroveFamily::KantoDeciduous {deciduous::width_ratio(coordinates,prototype)}
    else if family==GroveFamily::ParkBroadleaf {park_tree::dimensions(1.,1.,coordinates).1
        / park_tree::dimensions(1.,1.,prototype).1} else {1.}
}

pub(crate) fn instance_turn(family:GroveFamily,coordinates:[i32;2])->f32 {
    if family==GroveFamily::KantoDeciduous {deciduous::turn(coordinates)}
        else if family==GroveFamily::ParkBroadleaf {-park_tree::phase(coordinates)}
        else {variation(coordinates[0],coordinates[1]).turn}
}

pub(crate) fn instance_canopy_margin(family:GroveFamily,coordinates:[i32;2])->f32 {
    match family {
        GroveFamily::KantoDeciduous=>deciduous::canopy_margin(coordinates),
        GroveFamily::AncientForest=>1.25,
        GroveFamily::JohtoPine | GroveFamily::WinterPine=>0.75,
        GroveFamily::ParkBroadleaf=>2.0,
    }
}

pub(crate) fn grove_trunk_offset(coordinates: [i32; 2], spacing: f32) -> Vec2 {
    let v = variation(coordinates[0], coordinates[1]);
    // Break up rows visibly while leaving room for the fitted trunk flare
    // inside the original blocked plot.
    Vec2::new(v.offset[0], v.offset[1]) * spacing * 0.32
}

/// Restrained pigment patches within a crown, independent of tile boundaries.
/// Each complete leaf uses one sample so its lighting and solid shape remain clear.
pub(crate) fn leaf_pigment(point: Vec3, scale: f32) -> [f32;3] {
    let p = point / scale.max(0.001);
    let patch = ((p.dot(Vec3::new(0.83,0.57,-0.46))).sin()
        + (p.dot(Vec3::new(-0.39,1.17,0.71))+1.9).sin()*0.55
        + (p.dot(Vec3::new(2.13,-0.67,1.47))+0.8).sin()*0.18)/1.73;
    let warmth = (p.dot(Vec3::new(-0.71,0.31,1.09))+2.7).sin();
    [1.+patch*0.065+warmth*0.025,
     1.+patch*0.055,
     1.+patch*0.045-warmth*0.025]
}

/// Curved, tapered branch sections, continuous at their shared rings.
fn append_branch(mesh: &mut SurfaceMeshData, root: Vec3, tip: Vec3, radius: f32) {
    append_branch_with_detail(mesh, root, tip, radius, false);
}

fn append_branch_with_detail(
    mesh: &mut SurfaceMeshData,
    root: Vec3,
    tip: Vec3,
    radius: f32,
    distant: bool,
) {
    let axis = (tip - root).normalize();
    let side = axis.cross(Vec3::Z).normalize();
    let up = side.cross(axis).normalize();
    let first = mesh.positions.len() as u32;
    // Every row has the same ring directions and pigment. Preserve the
    // original arithmetic once per ring rather than repeating it per row.
    let rings: [(Vec3, [f32; 4]); 11] = std::array::from_fn(|ring| {
        let a=std::f32::consts::TAU*ring as f32/10.0;
        let normal=side*a.cos()+up*a.sin();
        let shade=0.75+0.25*normal.dot(Vec3::new(0.45,0.80,0.55).normalize()).max(0.0);
        (normal,[0.24*shade,0.15*shade,0.075*shade,1.0])
    });
    let step = if distant { 2 } else { 1 };
    let stride = 10 / step + 1;
    for row in (0..=6).step_by(step) {
        let t=row as f32/6.0;
        let center=root.lerp(tip,t)+Vec3::Y*(tip.y-root.y)*0.13*(std::f32::consts::PI*t).sin();
        let r=radius*(1.0-0.72*t);
        for ring in (0..=10).step_by(step) {
            let (normal, color) = rings[ring];
            mesh.positions.push((center+normal*r).to_array());
            mesh.normals.push(normal.to_array());
            mesh.uvs.push([0.0;2]);
            mesh.colors.push(color);
        }
    }
    for row in 0..6 / step {
        for ring in 0..10 / step {
            let a = first + (row * stride + ring) as u32;
            let b = a + stride as u32;
            mesh.indices.extend([a, b, a + 1, a + 1, b, b + 1]);
        }
    }
}

fn crown_color(normal: Vec3, tint: [f32; 3], variation: f32) -> [f32; 4] {
    let sun = normal.dot(Vec3::new(0.45, 0.80, 0.55).normalize()).max(0.0);
    let top = sun * sun * (3.0 - 2.0 * sun);
    // These are the formal park leaves. Give their shaded undersides actual
    // blue-green pigment; the scene lighting already provides canopy shadow.
    let cool = [0.022, 0.070, 0.042];
    let warm = [0.22, 0.36, 0.075];
    let mut color = [0.0; 4];
    for c in 0..3 {
        color[c] = (cool[c] + (warm[c] - cool[c]) * top) * tint[c] * variation;
    }
    color[3] = 1.0;
    color
}

fn crown_normal(direction: Vec3, radius: Vec3, lobe: f32, gradient: Vec3) -> Vec3 {
    ((direction-gradient/lobe)/radius).normalize()
}

const PARK_LEAF_EDGES: usize = 6;
const PARK_LEAF_BACK: usize = PARK_LEAF_EDGES + 1;
const PARK_LEAF_VERTICES: usize = PARK_LEAF_BACK * 2;
const PARK_LEAF_FACES: [[usize; 3]; PARK_LEAF_EDGES * 2] = {
    let mut faces = [[0; 3]; PARK_LEAF_EDGES * 2];
    let mut edge = 0;
    while edge < PARK_LEAF_EDGES {
        let a = edge + 1;
        let b = (edge + 1) % PARK_LEAF_EDGES + 1;
        faces[edge * 2] = [0, a, b];
        faces[edge * 2 + 1] = [PARK_LEAF_BACK, b + PARK_LEAF_BACK, a + PARK_LEAF_BACK];
        edge += 1;
    }
    faces
};

fn park_leaf(
    p: Vec3,
    u: Vec3,
    v: Vec3,
    n: Vec3,
    size: f32,
) -> ([Vec3; PARK_LEAF_VERTICES], [Vec3; PARK_LEAF_VERTICES]) {
    // Broad, cupped garden leaves retain a folded midrib and softly rounded
    // shoulders. Sharp route-leaf tips read as paper chips on these wide crowns.
    let mut vertices = [p; PARK_LEAF_VERTICES];
    vertices[0] = p + u * size * 0.05 + n * size * 0.18;
    // A shallow underside meets the same perimeter as the folded upper face.
    // This closed leaf stays visible from behind with ordinary back-face
    // culling, including at the exposed edges of the spreading crown.
    vertices[PARK_LEAF_BACK] = p + u * size * 0.05 - n * size * 0.06;
    for edge in 0..PARK_LEAF_EDGES {
        let angle = std::f32::consts::TAU * edge as f32 / PARK_LEAF_EDGES as f32;
        let along = angle.cos();
        let across = angle.sin();
        let curl = along * along * 0.08 - across * across * 0.035;
        vertices[edge + 1] = p + (u * along * 1.35 + v * across + n * curl) * size;
        vertices[PARK_LEAF_BACK + edge + 1] = vertices[edge + 1];
    }
    // The rim is a sharp blade edge. Sharing its upper and lower normals
    // rounds each leaf into a puffy scale even though its geometry is thin.
    let mut normals = [Vec3::ZERO; PARK_LEAF_VERTICES];
    for [a, b, c] in PARK_LEAF_FACES {
        let face = (vertices[b] - vertices[a]).cross(vertices[c] - vertices[a]);
        for i in [a, b, c] {
            normals[i] += face;
        }
    }
    for normal in &mut normals {
        *normal = normal.normalize();
    }
    (vertices, normals)
}

fn town_crown(mesh: &mut SurfaceMeshData, center: Vec3, radius: Vec3, phase: f32, tint: [f32; 3]) {
    // Garden leaves remain their own broadleaf family, with no solid crown shell.
    // Four leaves grow from each spray. All 1,200 leaves retain their real
    // upper and lower surfaces; the shared runtime library reuses this full
    // geometry. There is no enclosing foliage shell.
    let sprays = 300;
    for spray in 0..sprays {
        let scatter = |salt: u32| {
            let mut h = (spray as u32).wrapping_add(salt).wrapping_mul(0x9e3779b9);
            h = (h ^ (h >> 16)).wrapping_mul(0x85ebca6b);
            h ^= h >> 13;
            (h & 0x00ffffff) as f32 / 16777216.0
        };
        let y = 1.0 - 2.0 * (spray as f32 + 0.5) / sprays as f32;
        let b = spray as f32 * 2.399963 + phase + (scatter(17) - 0.5) * 0.30;
        let a = y.acos();
        let d = Vec3::new(
            (1.0 - y * y).sqrt() * b.cos(),
            y,
            (1.0 - y * y).sqrt() * b.sin(),
        );
        let (lobe, gradient) = park_tree::crown_shape(a, b, phase);
        let surface_normal = crown_normal(d, radius, lobe, gradient);
        let side = surface_normal
            .cross(if surface_normal.y.abs() > 0.9 {
                Vec3::Z
            } else {
                Vec3::Y
            })
            .normalize();
        let turn = scatter(71) * std::f32::consts::TAU;
        let axis = side * turn.cos() + surface_normal.cross(side) * turn.sin();
        let across = surface_normal.cross(axis);
        let depth = if spray % 3 == 0 {
            0.58 + 0.25 * scatter(401)
        } else {
            0.90 + 0.12 * scatter(401)
        };
        let size =
            radius.x.min(radius.z) * 0.875 * (0.087 + 0.019 * (spray as f32 * 1.37 + phase).sin());
        let root = center + d * radius * lobe * depth;
        let variation = (0.91 + 0.16 * scatter(137)) * (0.94 + 0.06 * depth);
        for leaf in 0..4 {
            let pair = (leaf / 2) as f32;
            let hand = if leaf % 2 == 0 { -1. } else { 1. };
            let n = (surface_normal
                + axis * (scatter(211) * 2. - 1.) * 0.30
                + across * (hand * 0.20 + (scatter(307) * 2. - 1.) * 0.18))
                .normalize();
            let direction = axis * 0.45 + across * hand * 0.90;
            let u = (direction - n * direction.dot(n)).normalize();
            let v = n.cross(u);
            let p = root
                + axis * ((pair - 0.5) * size * 1.7)
                + across * (hand * size * 0.55)
                + surface_normal * (pair * size * 0.12);
            let (vertices, normals) = park_leaf(p, u, v, n, size * (1. - pair * 0.09));
            let start = mesh.positions.len() as u32;
            for (index, point) in vertices.into_iter().enumerate() {
                let n = normals[index];
                mesh.positions.push(point.to_array());
                mesh.normals.push(n.to_array());
                mesh.uvs.push([0.; 2]);
                mesh.colors.push(crown_color(
                    n,
                    tint,
                    variation * if index == 0 { 1.07 } else { 0.97 },
                ));
            }
            for face in PARK_LEAF_FACES {
                mesh.indices.extend(face.map(|i| start + i as u32));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::new_bark_models::{self, ModelKind};
    #[test]
    fn broadleaf_crowns_turn_continuously_while_other_grove_poses_stay_exact() {
        let mut turns=std::collections::BTreeSet::new();
        let mut margins=Vec::new();
        for x in -32..32 {
            let at=[x,14];
            let angle=instance_turn(GroveFamily::KantoDeciduous,at);
            assert_eq!(angle,instance_turn(GroveFamily::KantoDeciduous,at));
            turns.insert(angle.to_bits());
            margins.push(instance_canopy_margin(GroveFamily::KantoDeciduous,at));
            for family in [GroveFamily::JohtoPine,GroveFamily::AncientForest] {
                assert_eq!(instance_turn(family,at),variation(at[0],at[1]).turn);
                assert_eq!(instance_canopy_margin(family,at),
                    if family==GroveFamily::JohtoPine {0.75}else{1.25});
            }
        }
        assert!(turns.len()>50,"tree silhouettes cannot repeat only four quarter turns");
        assert!(margins.iter().any(|&m|m==0.75) && margins.iter().any(|&m|m>1.5),
            "young crowns keep their margin while mature crowns can spread");
    }
    #[test]
    fn route_trees_mix_young_crowns_and_tall_leaders_without_changing_ancient_scale() {
        let mut low = f32::INFINITY;
        let mut high = f32::NEG_INFINITY;
        let mut leader_low = f32::INFINITY;
        let mut leader_high = f32::NEG_INFINITY;
        let mut young = 0;
        let mut mature = 0;
        for z in -32..32 {for x in -32..32 {
            let coordinates = [x,z];
            let height = non_ancient_height_factor(coordinates);
            assert!((0.5..=1.500001).contains(&height));
            assert_eq!(height,non_ancient_height_factor(coordinates));
            low=low.min(height); high=high.max(height);
            let leader = pine::stem_height(16., coordinates, [false;4]);
            leader_low = leader_low.min(leader);
            leader_high = leader_high.max(leader);
            if leader < 16. * 4. { young += 1; }
            if leader > 16. * 12. { mature += 1; }
            let prototype=[4,8];
            for neighbors in [[false;4],[true;4]] {
                let ratio=instance_vertical_ratio(GroveFamily::JohtoPine,coordinates,prototype,neighbors);
                assert!((pine::stem_height(1.,prototype,neighbors)*ratio
                    -pine::stem_height(1.,coordinates,neighbors)).abs()<0.00001,
                    "shared prototypes must apply the placement's height once");
                assert_eq!(instance_vertical_ratio(GroveFamily::AncientForest,coordinates,prototype,neighbors),
                    variation(x,z).height/variation(prototype[0],prototype[1]).height,
                    "ordinary-tree variation must leave ancient scaling unchanged");
            }
        }}
        assert!(low<0.52 && high>1.48,"ordinary trees must use the requested height range");
        assert!(leader_high > leader_low * 5.5,
            "actual route trees must span young crowns through mature leaders");
        assert!(young > 600 && mature > 400,
            "both lower crowns and tall leaders must recur throughout a route block");
    }
    #[test]
    fn foliage_pigment_varies_inside_one_crown_without_strong_color_swings() {
        let samples = (0..64).map(|i| leaf_pigment(
            Vec3::new((i%8) as f32, (i/8) as f32, 1.7),2.)).collect::<Vec<_>>();
        assert!(samples.iter().flatten().all(|v| (0.90..=1.10).contains(v)));
        let low = samples.iter().map(|c|c[1]).fold(f32::INFINITY,f32::min);
        let high = samples.iter().map(|c|c[1]).fold(f32::NEG_INFINITY,f32::max);
        assert!(high-low > 0.035, "a single crown must contain multiple subtle pigment patches");
        assert_eq!(leaf_pigment(Vec3::ONE,2.),leaf_pigment(Vec3::ONE,2.));
    }
    #[test]
    fn forest_drawing_preserves_ancient_and_small_headbutt_identities() {
        assert_eq!(GroveFamily::from_drawing("forest", 4, 4), GroveFamily::AncientForest);
        assert_eq!(GroveFamily::from_drawing("forest", 2, 2), GroveFamily::JohtoPine);
        assert_eq!(GroveFamily::from_drawing("johto", 2, 4), GroveFamily::JohtoPine);
    }

    #[test]
    fn route_trees_keep_authored_trunk_plots_and_mature_leaders_clear_roofs() {
        let roof = [ModelKind::House, ModelKind::PlayerHouse, ModelKind::Lab,
            ModelKind::Pokecenter, ModelKind::Mart].into_iter().map(|kind| {
                let model = new_bark_models::model(kind);
                // Buildings use two source tiles per authored model unit.
                (model.max[1] - model.min[1]) * 16. * 2.
            }).fold(0., f32::max);
        for coordinates in [[0,0], [4,8], [-12,4], [6,8], [13,6], [29,4]] {
            for neighbors in [[false;4], [true;4], [true,false,true,false]] {
                let mut tree = TerrainMeshData::default();
                append_grove_tree(&mut tree, [0.,32.,0.,32.], 3.,16.,
                    coordinates,neighbors);
                let top = tree.solid.positions.iter().map(|p| p[1]).fold(3.,f32::max);
                if pine::stem_height(16.,coordinates,neighbors) > 16. * 12. {
                    assert!(top > 3. + roof * 1.10,
                        "mature leader must stand above town roofs: {coordinates:?}, {top}, roof {roof}");
                }
                assert!(top >= 3. + pine::stem_height(16.,coordinates,neighbors),
                    "crown fitting must retain the full leader height");
                assert!(tree.solid.positions[..13].iter().all(|p|
                    p[1] == 3. && p[0] > 0. && p[0] < 32. && p[2] > 0. && p[2] < 32.),
                    "the rooted flare must remain on its authored plot");
            }
        }
    }
    #[test]
    fn grove_variation_is_stable_diverse_and_bounded() {
        let mut counts = [0; 3];
        let mut turns = [0; 4];
        for z in -10..10 {
            for x in -10..10 {
                let v = variation(x, z);
                assert_eq!(v, variation(x, z));
                counts[v.index] += 1;
                turns[(v.turn / std::f32::consts::FRAC_PI_2).round() as usize] += 1;
                assert!((0.90..=1.601).contains(&v.height));
                assert!((0.84..=1.001).contains(&v.width));
            }
        }
        assert!(counts.iter().all(|n| *n > 40));
        assert!(turns.iter().all(|n| *n > 50));
    }
}
