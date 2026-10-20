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
pub(crate) use park_tree::append_park_tree;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GroveFamily { JohtoPine, AncientForest, KantoDeciduous }
impl GroveFamily {
    pub(crate) fn from_drawing(tileset: &str, width: usize, height: usize) -> Self {
        // Forest's small 2x2 Headbutt drawing and 4x4 ancient canopy are
        // different authored objects. Map identity must never conflate them.
        if tileset == "forest" && width == 2 && height == 2 {
            Self::JohtoPine
        } else { Self::from_tileset(tileset) }
    }
    pub(crate) fn from_tileset(tileset: &str) -> Self {
        match tileset {"forest"=>Self::AncientForest,"kanto"=>Self::KantoDeciduous,_=>Self::JohtoPine}
    }
    pub(crate) fn label(self) -> &'static str {
        match self { Self::JohtoPine => "johto/tree", Self::AncientForest => "forest/ancient_tree", Self::KantoDeciduous => "world-exterior/kanto_canopy" }
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
        GroveFamily::JohtoPine => pine::stem_height(1.,coordinates,neighbors)
            / pine::stem_height(1.,prototype,neighbors),
        GroveFamily::AncientForest => variation(coordinates[0],coordinates[1]).height
            / variation(prototype[0],prototype[1]).height,
        GroveFamily::KantoDeciduous => deciduous::height(1.,coordinates)/deciduous::height(1.,prototype),
    }
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
        GroveFamily::AncientForest => ancient::append(mesh,bounds,base,tile,coordinates,neighbors),
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
        GroveFamily::AncientForest => ancient::append_canopy(mesh,bounds,base,tile,coordinates,neighbors),
        GroveFamily::KantoDeciduous => deciduous::append_canopy(mesh,bounds,base,tile,coordinates,neighbors),
    }
    if mesh.solid.positions.len() > first {
        mesh.solid.cutaway_ranges.push(first..mesh.solid.positions.len());
    }
}



pub(crate) fn instance_width_ratio(family:GroveFamily,coordinates:[i32;2],prototype:[i32;2])->f32 {
    if family==GroveFamily::KantoDeciduous {deciduous::width_ratio(coordinates,prototype)} else {1.}
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
    let axis=(tip-root).normalize();
    let side=axis.cross(Vec3::Z).normalize();
    let up=side.cross(axis).normalize();
    let first=mesh.positions.len() as u32;
    // Every row has the same ring directions and pigment. Preserve the
    // original arithmetic once per ring rather than repeating it per row.
    let rings: [(Vec3, [f32; 4]); 11] = std::array::from_fn(|ring| {
        let a=std::f32::consts::TAU*ring as f32/10.0;
        let normal=side*a.cos()+up*a.sin();
        let shade=0.75+0.25*normal.dot(Vec3::new(0.45,0.80,0.55).normalize()).max(0.0);
        (normal,[0.24*shade,0.15*shade,0.075*shade,1.0])
    });
    for row in 0..=6 {
        let t=row as f32/6.0;
        let center=root.lerp(tip,t)+Vec3::Y*(tip.y-root.y)*0.13*(std::f32::consts::PI*t).sin();
        let r=radius*(1.0-0.72*t);
        for ring in 0..=10 {
            let (normal, color) = rings[ring];
            mesh.positions.push((center+normal*r).to_array());
            mesh.normals.push(normal.to_array());
            mesh.uvs.push([0.0;2]);
            mesh.colors.push(color);
        }
    }
    for row in 0..6 {for ring in 0..10 {
        let a=first+row*11+ring;let b=a+11;
        mesh.indices.extend([a,b,a+1,a+1,b,b+1]);
    }}
}

fn crown_color(normal: Vec3, tint: [f32; 3], variation: f32) -> [f32; 4] {
    let sun = normal.dot(Vec3::new(0.45, 0.80, 0.55).normalize()).max(0.0);
    let top = sun * sun * (3.0 - 2.0 * sun);
    // Vertex colors enter the linear render pipeline. Keep the shaded core
    // dark enough after display conversion for leaf clusters to catch the sun.
    let cool = [0.008, 0.032, 0.020];
    let warm = [0.24, 0.39, 0.060];
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

fn park_leaf(p: Vec3, u: Vec3, v: Vec3, n: Vec3, size: f32) -> ([Vec3; 9], [Vec3; 9]) {
    // Broad, cupped garden leaves retain a folded midrib and softly rounded
    // shoulders. Sharp route-leaf tips read as paper chips on these wide crowns.
    let mut vertices = [p; 9];
    vertices[0] = p + u * size * 0.05 + n * size * 0.18;
    for edge in 0..8 {
        let angle = std::f32::consts::TAU * edge as f32 / 8.;
        let along = angle.cos();
        let across = angle.sin();
        let curl = along * along * 0.08 - across * across * 0.035;
        vertices[edge + 1] = p + (u * along * 1.35 + v * across * 0.90 + n * curl) * size;
    }
    let mut normals = [Vec3::ZERO; 9];
    for edge in 0..8 {
        let a = edge + 1;
        let b = (edge + 1) % 8 + 1;
        let face = (vertices[a] - vertices[0]).cross(vertices[b] - vertices[0]);
        for i in [0, a, b] { normals[i] += face; }
    }
    for normal in &mut normals { *normal = normal.normalize(); }
    (vertices, normals)
}

fn town_crown(mesh: &mut SurfaceMeshData, center: Vec3, radius: Vec3, phase: f32, tint: [f32;3]) {
    // Garden leaves remain their own broadleaf family, with no solid crown shell.
    let leaves = 960;
    for i in 0..leaves {
        let scatter = |salt: u32| {
            let mut h=(i as u32).wrapping_add(salt).wrapping_mul(0x9e3779b9);
            h=(h^(h>>16)).wrapping_mul(0x85ebca6b); h^=h>>13;
            (h & 0x00ffffff) as f32 / 16777216.0
        };
        let y=1.0-2.0*(i as f32+0.5)/leaves as f32;
        let b=scatter(17)*std::f32::consts::TAU+phase; let a=y.acos();
        let d=Vec3::new((1.0-y*y).sqrt()*b.cos(),y,(1.0-y*y).sqrt()*b.sin());
        let (lobe, gradient)=park_tree::crown_shape(a,b,phase);
        let surface_normal=crown_normal(d,radius,lobe,gradient);
        let side=surface_normal.cross(if surface_normal.y.abs()>0.9 {Vec3::Z} else {Vec3::Y}).normalize();
        let n=(surface_normal+side*(scatter(211)*2.-1.)*0.28
            +surface_normal.cross(side)*(scatter(307)*2.-1.)*0.20).normalize();
        let tangent=n.cross(if n.y.abs()>0.9 {Vec3::Z} else {Vec3::Y}).normalize();
        let turn=scatter(71)*std::f32::consts::TAU;
        let u=tangent*turn.cos()+n.cross(tangent)*turn.sin();
        let v=n.cross(u);
        let size=radius.x.min(radius.z)*0.70*(0.105+0.035*(i as f32*1.37+phase).sin());
        let p=center+d*radius*lobe+surface_normal*size*0.10;
        let (vertices, normals)=park_leaf(p,u,v,n,size);
        let start=mesh.positions.len() as u32;
        let variation=0.88+0.20*scatter(137);
        for (index,point) in vertices.into_iter().enumerate() {
            let n=normals[index];
            mesh.positions.push(point.to_array());mesh.normals.push(n.to_array());mesh.uvs.push([0.;2]);
            mesh.colors.push(crown_color(n,tint,variation*if index==0 {1.07} else {0.97}));
        }
        for edge in 0..8 {mesh.indices.extend([start,start+1+edge,start+1+(edge+1)%8]);}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::new_bark_models::{self, ModelKind};
    #[test]
    fn ordinary_tree_height_uses_fifty_percent_variation_without_changing_ancient_scale() {
        let mut low = f32::INFINITY;
        let mut high = f32::NEG_INFINITY;
        let mut leader_low = f32::INFINITY;
        let mut leader_high = f32::NEG_INFINITY;
        for z in -32..32 {for x in -32..32 {
            let coordinates = [x,z];
            let height = non_ancient_height_factor(coordinates);
            assert!((0.5..=1.500001).contains(&height));
            assert_eq!(height,non_ancient_height_factor(coordinates));
            low=low.min(height); high=high.max(height);
            let leader = pine::stem_height(16., coordinates, [false;4]);
            leader_low = leader_low.min(leader);
            leader_high = leader_high.max(leader);
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
        assert!(leader_high > leader_low * 1.9,
            "the rendered leaders must retain the wide range, rather than compressing it with a height floor");
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
    fn mature_trees_clear_house_roofs_and_keep_authored_trunk_plots() {
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
                assert!(top > 3. + roof * 1.10,
                    "generated pine must stand above town roofs: {coordinates:?}, {top}, roof {roof}");
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
