//! Stable per-placement variation. Camera movement never changes a tree.
use crate::{
    mesh::{SurfaceMeshData, TerrainMeshData},
    new_bark_models::{self, Model, ModelKind},
};
use bevy::prelude::*;
use std::sync::OnceLock;

#[path = "foliage/park_tree.rs"]
mod park_tree;
pub(crate) use park_tree::append_park_tree;

pub(crate) const VARIANT_PATHS: [&str; 4] = [
    "models/johto/tree_variants/tree_2.mesh.json",
    "models/johto/tree_variants/tree_3.mesh.json",
    "models/johto/tree_variants/tree_4.mesh.json",
    "models/johto/tree_variants/tree_5.mesh.json",
];

fn variant(kind: ModelKind, index: usize) -> Option<&'static Model> {
    static VARIANTS: [OnceLock<Option<Model>>; 4] = [const { OnceLock::new() }; 4];
    if index == 0 || kind == ModelKind::TreeLod { return new_bark_models::required_tree_model(kind); }
    VARIANTS[index - 1].get_or_init(||
        crate::open_models::load_required(VARIANT_PATHS[index - 1], |source| Model::parse(source))).as_ref()
}

pub(crate) fn world_tree_available(coordinates: [i32; 2]) -> bool {
    variant(ModelKind::Tree, variation(coordinates[0], coordinates[1]).index).is_some()
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
        index: value as usize % 5,
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

/// Mature world trees use physical world scale, independently of source drawing height.
/// Crowns overlap within groves; trunks remain inside their authored blocked plots.
pub(crate) fn append_world_tree(
    mesh: &mut TerrainMeshData,
    kind: ModelKind,
    bounds: [f32; 4],
    base: f32,
    tile_height: f32,
    coordinates: [i32; 2],
    neighbors: [bool; 4],
) -> bool {
    let v = variation(coordinates[0], coordinates[1]);
    let spacing = (bounds[1] - bounds[0]).min(bounds[3] - bounds[2]);
    let center = [(bounds[0] + bounds[1]) * 0.5, (bounds[2] + bounds[3]) * 0.5];
    let density = neighbors.iter().filter(|&&n| n).count() as f32 / 4.0;
    let offset = [v.offset[0]*spacing*0.30*density,v.offset[1]*spacing*0.30*density];
    let crown=spacing*(1.0+density*1.45);
    let fitted=[center[0]+offset[0]-crown*0.5,center[0]+offset[0]+crown*0.5,
        center[1]+offset[1]-crown*0.5,center[1]+offset[1]+crown*0.5];
    append_tree_impl(mesh,kind,fitted,base,tile_height*10.0,coordinates,
        (density>0.0).then_some(Vec3::new(center[0]+offset[0],base,center[1]+offset[1])))
}

pub(crate) fn append_tree(
    mesh: &mut TerrainMeshData,
    kind: ModelKind,
    bounds: [f32; 4],
    base: f32,
    height: f32,
    coordinates: [i32; 2],
) -> bool {
    append_tree_impl(mesh, kind, bounds, base, height, coordinates, None)
}

/// One branching broadleaf tree for ordinary towns and routes. Placement,
/// neighbor density and stable source coordinates vary a shared asset family.
pub(crate) fn append_grove_tree(mesh: &mut TerrainMeshData, bounds: [f32;4], base: f32,
    tile: f32, coordinates: [i32;2], neighbors: [bool;4]) {
    append_broadleaf_tree(mesh, bounds, base, tile, coordinates, neighbors);
}

pub(crate) fn grove_trunk_offset(coordinates: [i32; 2], spacing: f32) -> Vec2 {
    let v = variation(coordinates[0], coordinates[1]);
    Vec2::new(v.offset[0], v.offset[1]) * spacing * 0.14
}

fn append_broadleaf_tree(mesh: &mut TerrainMeshData, bounds: [f32;4], base: f32,
    tile: f32, coordinates: [i32;2], neighbors: [bool;4]) {
    if mesh.solid.positions.is_empty() {
        let (vertices, indices) = grove_geometry_size(coordinates, neighbors);
        mesh.solid.positions.reserve_exact(vertices);
        mesh.solid.normals.reserve_exact(vertices);
        mesh.solid.uvs.reserve_exact(vertices);
        mesh.solid.colors.reserve_exact(vertices);
        mesh.solid.indices.reserve_exact(indices);
    }
    let v=variation(coordinates[0],coordinates[1]);
    let interior=neighbors.iter().all(|&n| n);
    // A continuous interior grove gets one broad crown per four source plots.
    // Ground ownership is still handled for every plot by the caller.
    if interior && (coordinates[0].rem_euclid(4)!=0 || coordinates[1].rem_euclid(4)!=0) {
        let first=mesh.solid.positions.len();
        crate::understory::append(&mut mesh.solid,bounds,base,tile,coordinates);
        mesh.solid.cutaway_ranges.push(first..mesh.solid.positions.len());
        return;
    }
    let spacing=(bounds[1]-bounds[0]).min(bounds[3]-bounds[2]);
    let center=Vec3::new((bounds[0]+bounds[1])*0.5,base,(bounds[2]+bounds[3])*0.5)
        + { let offset=grove_trunk_offset(coordinates,spacing);
            Vec3::new(offset.x,0.0,offset.y) };
    let first=mesh.solid.positions.len();
    let height=tile*((if interior {6.4} else {4.8})+v.height*0.8);
    // Leave space inside boundary plots for the wider oval leaf silhouettes.
    let density=neighbors.iter().filter(|&&n| n).count() as f32/4.0;
    let width=spacing*if interior {2.0} else {0.90+density*1.3};
    let trunk_radius=spacing*0.125;
    // A tapered trunk with gentle ridges, rather than high-contrast birch rings.
    let start=mesh.solid.positions.len() as u32;
    for row in 0..=6 {for side in 0..=16 {
        let t=row as f32/6.0;let a=std::f32::consts::TAU*side as f32/16.0;
        let flare=1.0+0.7*(1.0-t).powi(5)*(0.65+0.35*(a*4.0+v.turn).cos());
        let r=trunk_radius*(1.0-0.48*t)*(1.0+0.08*(a*5.0).cos())*flare;
        let bend=Vec3::new(v.turn.cos(),0.0,v.turn.sin())*spacing*0.13*t*t;
        let p=center+Vec3::new(a.cos()*r,t*height*0.50,a.sin()*r)+bend;
        mesh.solid.positions.push(p.to_array());
        mesh.solid.normals.push(Vec3::new(a.cos(),0.04,a.sin()).normalize().to_array());
        mesh.solid.uvs.push([0.0;2]);
        let groove = 0.83 + 0.17 * (a * 5.0 + v.turn + t * 0.8).cos();
        let light=(0.65+0.35*(a.cos()*0.3+a.sin()*0.7).max(0.0)) * groove;
        mesh.solid.colors.push([0.24*light,0.15*light,0.075*light,1.0]);
    }}
    for row in 0..6 {for side in 0..16 {
        let a=start+(row*17+side) as u32;let b=a+17;
        mesh.solid.indices.extend([a,b,a+1,a+1,b,b+1]);
    }}
    let phase=v.turn + v.height*5.31;
    {
        for branch in 0..4 {
            let angle=phase+branch as f32*2.399963;
            let start=center+Vec3::Y*height*(0.25+branch as f32*0.045);
            let end=center+Vec3::new(angle.cos()*spacing*0.33,
                height*(0.48+branch as f32*0.025), angle.sin()*spacing*0.33);
            append_branch(&mut mesh.solid,start,end,trunk_radius*(0.60-branch as f32*0.075));
            // Fork the supporting limbs beneath the leaf masses, instead
            // of ending every branch as a single spoke into the crown.
            let fork=start.lerp(end,0.62)+Vec3::Y*(end.y-start.y)*0.10;
            for side in [-1.0,1.0] {
                let turn=angle+side*0.38;
                let tip=center+Vec3::new(turn.cos()*spacing*0.33,
                    height*(0.54+branch as f32*0.025),turn.sin()*spacing*0.33);
                append_branch(&mut mesh.solid,fork,tip,trunk_radius*0.19);
            }
        }
        // One connected, asymmetrical crown reads as a branching broadleaf
        // tree. Separate ellipsoids made the grove look like stacked balls.
        let offset=Vec3::new(phase.cos()*width*0.035,height*0.54,phase.sin()*width*0.045);
        // Carry the lower leaf masses around the supporting forks. Preserve
        // a spread limit, but avoid a thin crown perched above a bare trunk.
        let radius=Vec3::new(width*0.62,(height*0.26).min(width*0.42),width*0.58);
        town_crown(&mut mesh.solid,center+offset,radius,phase,v.tint,CrownStyle::Woodland);
    }
    crate::understory::append(&mut mesh.solid,bounds,base,tile,coordinates);
    mesh.solid.cutaway_ranges.push(first..mesh.solid.positions.len());
}

fn grove_geometry_size(coordinates: [i32; 2], neighbors: [bool; 4]) -> (usize, usize) {
    // Understory: 16x7 rosette leaves and 8x5x11 fern leaves, each four
    // faceted triangles. A canopy adds trunk, twelve branches and 1,920 leaves.
    let understory = (16 * 7 + 8 * 5 * 11) * 12;
    if neighbors.iter().all(|&n| n)
        && (coordinates[0].rem_euclid(4) != 0 || coordinates[1].rem_euclid(4) != 0)
    {
        (understory, understory)
    } else {
        (understory + 7 * 17 + 12 * 7 * 11 + 13 * 25 + 1920 * 9,
         understory + 6 * 16 * 6 + 12 * 6 * 10 * 6 + 22 * 24 * 3 + 1920 * 8 * 3)
    }
}

/// Curved, tapered branch sections, continuous at their shared rings.
fn append_branch(mesh: &mut SurfaceMeshData, root: Vec3, tip: Vec3, radius: f32) {
    let axis=(tip-root).normalize();
    let side=axis.cross(Vec3::Z).normalize();
    let up=side.cross(axis).normalize();
    let first=mesh.positions.len() as u32;
    for row in 0..=6 {
        let t=row as f32/6.0;
        let center=root.lerp(tip,t)+Vec3::Y*(tip.y-root.y)*0.13*(std::f32::consts::PI*t).sin();
        let r=radius*(1.0-0.72*t);
        for ring in 0..=10 {
            let a=std::f32::consts::TAU*ring as f32/10.0;
            let normal=side*a.cos()+up*a.sin();
            mesh.positions.push((center+normal*r).to_array());
            mesh.normals.push(normal.to_array());
            mesh.uvs.push([0.0;2]);
            let shade=0.75+0.25*normal.dot(Vec3::new(0.45,0.80,0.55).normalize()).max(0.0);
            mesh.colors.push([0.24*shade,0.15*shade,0.075*shade,1.0]);
        }
    }
    for row in 0..6 {for ring in 0..10 {
        let a=first+row*11+ring;let b=a+11;
        mesh.indices.extend([a,b,a+1,a+1,b,b+1]);
    }}
}

fn crown_color(normal: Vec3, tint: [f32; 3], variation: f32, leaf: bool) -> [f32; 4] {
    let sun = normal.dot(Vec3::new(0.45, 0.80, 0.55).normalize()).max(0.0);
    let top = sun * sun * (3.0 - 2.0 * sun);
    // Vertex colors enter the linear render pipeline. Keep the shaded core
    // dark enough after display conversion for leaf clusters to catch the sun.
    let cool = [0.008, 0.032, 0.020];
    let warm = if leaf { [0.24, 0.39, 0.060] } else { [0.065, 0.17, 0.032] };
    let mut color = [0.0; 4];
    for c in 0..3 {
        color[c] = (cool[c] + (warm[c] - cool[c]) * top) * tint[c] * variation;
    }
    color[3] = 1.0;
    color
}

// Return the radius and its tangent gradient on the unit sphere. The
// gradient lets both the crown skin and its leaves light the actual lobes.
fn crown_shape(a: f32, b: f32, phase: f32, woodland: bool) -> (f32, Vec3) {
    if woodland {
        let direction = Vec3::new(a.sin()*b.cos(), a.cos(), a.sin()*b.sin());
        let mut mass = 0.78;
        let mut gradient = Vec3::ZERO;
        for cluster in 0..5 {
            let turn = phase+cluster as f32*2.399963;
            let y = if cluster<3 {0.12+0.15*(phase+cluster as f32).sin()}
                else {0.72+0.08*(phase*1.7+cluster as f32).sin()};
            let spread = (1.0-y*y).sqrt();
            let axis = Vec3::new(spread*turn.cos(), y, spread*turn.sin());
            let dot = direction.dot(axis);
            let dominance = 0.5 + 0.5*(phase*1.37+cluster as f32*0.91).sin();
            let weight = if cluster<3 {0.32+dominance*0.18} else {0.40+dominance*0.18};
            let lobe = weight*((dot-1.0)*9.0).exp();
            mass += lobe;
            gradient += (axis-direction*dot)*(9.0*lobe);
        }
        let ripple = b*9.0+a*7.0+phase;
        mass += 0.025*ripple.sin()*a.sin();
        if a.sin().abs()>0.001 {
            let along_a = Vec3::new(a.cos()*b.cos(), -a.sin(), a.cos()*b.sin());
            let along_b = Vec3::new(-b.sin(), 0.0, b.cos());
            gradient += along_a*(0.025*(7.0*ripple.cos()*a.sin()+ripple.sin()*a.cos()))
                + along_b*(0.225*ripple.cos());
        } else {
            gradient = Vec3::ZERO;
        }
        (mass, gradient)
    } else {
        (1.0+0.04*(b*5.0+phase).sin()*a.sin().powi(2)
            +0.025*(a*6.0+phase).sin(), Vec3::ZERO)
    }
}

fn crown_normal(direction: Vec3, radius: Vec3, lobe: f32, gradient: Vec3) -> Vec3 {
    ((direction-gradient/lobe)/radius).normalize()
}

fn pointed_leaf(p: Vec3, u: Vec3, v: Vec3, n: Vec3, size: f32) -> ([Vec3; 9], [Vec3; 9]) {
    // Keep nearly the oval's area, but give it a pointed blade and a folded
    // midrib. The two ends curl unequally instead of forming a flat decal.
    let outline = [(1.80,0.),(0.90,0.55),(0.,0.75),(-0.90,0.55),
        (-1.80,0.),(-0.90,-0.55),(0.,-0.75),(0.90,-0.55)];
    let mut vertices = [p; 9];
    vertices[0] = p + u*size*0.08 + n*size*0.24;
    for (i, (along, across)) in outline.into_iter().enumerate() {
        let curl = (along/1.80_f32).powi(2) * if along>0. {0.12} else {-0.08};
        vertices[i+1] = p + (u*along + v*across + n*curl)*size;
    }
    let mut normals = [Vec3::ZERO; 9];
    for edge in 0..8 {
        let a = edge+1;
        let b = (edge+1)%8+1;
        let face = (vertices[a]-vertices[0]).cross(vertices[b]-vertices[0]);
        for i in [0,a,b] { normals[i] += face; }
    }
    for normal in &mut normals { *normal = normal.normalize(); }
    (vertices, normals)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CrownStyle { Woodland, Park }

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

fn town_crown(mesh: &mut SurfaceMeshData, center: Vec3, radius: Vec3, phase: f32, tint: [f32;3], style: CrownStyle) {
    // Garden boughs use leaves alone. A closed inner shell filled every
    // gap and cast one solid blob regardless of the leaf geometry outside it.
    if style != CrownStyle::Park {
    let rows = 12;
    let sides = 24;
    let first=mesh.positions.len() as u32;
    for row in 0..=rows {for side in 0..=sides {
        let a=std::f32::consts::PI*row as f32/rows as f32;
        let b=std::f32::consts::TAU*side as f32/sides as f32;
        let d=Vec3::new(a.sin()*b.cos(),a.cos(),a.sin()*b.sin());
        let (lobe, gradient)=crown_shape(a,b,phase,true);
        let n=crown_normal(d,radius,lobe,gradient);
        let core = 0.92;
        mesh.positions.push((center+d*radius*lobe*core).to_array());mesh.normals.push(n.to_array());mesh.uvs.push([0.0;2]);
        let leaf=1.0+0.08*(b*7.0+a*9.0+phase).sin()*a.sin();
        mesh.colors.push(crown_color(n, tint, leaf, false));
    }}
    for row in 0..rows {for side in 0..sides {
        let a=first+(row*(sides+1)+side) as u32;let b=a+(sides+1) as u32;
        if row>0 {mesh.indices.extend([a,a+1,b]);}
        if row+1<rows {mesh.indices.extend([a+1,b+1,b]);}
    }}
    }
    // Small folded leaves break up the smooth crown masses at game scale.
    // Stable spherical sampling avoids visible latitude rows or camera cards.
    let leaves=if style == CrownStyle::Park {960} else {1920};
    for i in 0..leaves {
        let scatter = |salt: u32| {
            let mut h=(i as u32).wrapping_add(salt).wrapping_mul(0x9e3779b9);
            h=(h^(h>>16)).wrapping_mul(0x85ebca6b);
            h^=h>>13;
            (h & 0x00ffffff) as f32 / 16777216.0
        };
        let y=1.0-2.0*(i as f32+0.5)/leaves as f32;
        let b=scatter(17)*std::f32::consts::TAU+phase;let a=y.acos();
        let d=Vec3::new((1.0-y*y).sqrt()*b.cos(),y,(1.0-y*y).sqrt()*b.sin());
        let (lobe, gradient)=if style == CrownStyle::Park {
            park_tree::crown_shape(a, b, phase)
        } else { crown_shape(a,b,phase,true) };
        let n=crown_normal(d,radius,lobe,gradient);
        let surface_normal = n;
        // Garden leaves grow at unequal angles around their boughs. A single
        // tangent shell made the deeper park crowns look like solid lettuce.
        let n = if style == CrownStyle::Park {
            let side = n.cross(if n.y.abs() > 0.9 { Vec3::Z } else { Vec3::Y }).normalize();
            (n + side * (scatter(211) * 2. - 1.) * 0.28
                + n.cross(side) * (scatter(307) * 2. - 1.) * 0.20).normalize()
        } else { n };
        let tangent=n.cross(if n.y.abs()>0.9 {Vec3::Z} else {Vec3::Y}).normalize();
        let turn=scatter(71)*std::f32::consts::TAU;
        let u=tangent*turn.cos()+n.cross(tangent)*turn.sin();
        // Leaf dimensions follow the canopy span, independently of its new
        // vertical depth; fuller branches must not inflate individual leaves.
        let leaf_scale = if style == CrownStyle::Park {
            radius.x.min(radius.z) * 0.70
        } else { radius.min_element() };
        let v=n.cross(u);let size=leaf_scale*
            (if style == CrownStyle::Park {0.105} else {0.075}
                +0.035*(i as f32*1.37+phase).sin());
        let p=center+d*radius*lobe+surface_normal*size*0.10;
        let (vertices, leaf_normals) = if style == CrownStyle::Park {
            park_leaf(p,u,v,n,size)
        } else { pointed_leaf(p,u,v,n,size) };
        let start=mesh.positions.len() as u32;
        let variation=if style == CrownStyle::Park {0.88+0.20*scatter(137)}
            else {0.80+0.32*scatter(137)};
        for (index,point) in vertices.into_iter().enumerate() {
            let leaf_normal=leaf_normals[index];
            mesh.positions.push(point.to_array());mesh.normals.push(leaf_normal.to_array());mesh.uvs.push([0.0;2]);
            let edge=if index==0 {1.07} else {0.97};
            mesh.colors.push(crown_color(leaf_normal, tint, variation*edge, true));
        }
        for edge in 0..8 {mesh.indices.extend([start,start+1+edge,start+1+(edge+1)%8]);}
    }
}

fn append_tree_impl(
    mesh: &mut TerrainMeshData,
    kind: ModelKind,
    bounds: [f32; 4],
    base: f32,
    height: f32,
    coordinates: [i32; 2],
    root_anchor: Option<Vec3>,
) -> bool {
    let v = variation(coordinates[0], coordinates[1]);
    let Some(tree) = variant(kind, v.index) else { return false; };
    let mut center = Vec3::new(
        (bounds[0] + bounds[1]) * 0.5,
        base,
        (bounds[2] + bounds[3]) * 0.5,
    );
    // Tree plots are square; quarter turns retain the occupied source footprint.
    let side = (bounds[1] - bounds[0]).min(bounds[3] - bounds[2]) * v.width * 0.5;
    let margin = (bounds[1] - bounds[0]).min(bounds[3] - bounds[2]) * 0.5 - side;
    center.x += v.offset[0] * margin * 0.8;
    center.z += v.offset[1] * margin * 0.8;
    let fitted = [
        center.x - side,
        center.x + side,
        center.z - side,
        center.z + side,
    ];
    let scale = height * v.height / (tree.max[1] - tree.min[1]);
    let mut solid = SurfaceMeshData::default();
    tree.append_fitted(&mut solid, fitted, base, scale, None);
    let mut parts = Vec::new();
    tree.append_textured_fitted(&mut parts, fitted, base, scale);
    let rotate = Quat::from_rotation_y(v.turn);
    // Crown bounds are asymmetric, especially for split-trunk variants. Anchor
    // the actual low trunk geometry rather than assuming its root is centred.
    let mut root_min = Vec3::splat(f32::INFINITY);
    let mut root_max = Vec3::splat(f32::NEG_INFINITY);
    for &p in solid
        .positions
        .iter()
        .filter(|p| p[1] <= base + height * v.height * 0.02)
    {
        root_min = root_min.min(Vec3::from_array(p));
        root_max = root_max.max(Vec3::from_array(p));
    }
    let shift = root_anchor
        .filter(|_| root_min.is_finite())
        .map_or(Vec3::ZERO, |anchor| {
            let root = (root_min + root_max) * 0.5;
            let rotated = center + rotate * (root - center);
            Vec3::new(anchor.x - rotated.x, 0.0, anchor.z - rotated.z)
        });
    let finish = |surface: &mut SurfaceMeshData| {
        for position in &mut surface.positions {
            *position =
                (shift + center + rotate * (Vec3::from_array(*position) - center)).to_array();
        }
        let light = Vec3::new(0.45, 0.80, 0.55).normalize();
        for (normal, color) in surface.normals.iter_mut().zip(&mut surface.colors) {
            let original = Vec3::from_array(*normal);
            let rotated = rotate * original;
            // Reorient the existing key/fill bake along with the geometry.
            let shade = (0.62 + 0.38 * rotated.dot(light).max(0.0))
                / (0.62 + 0.38 * original.dot(light).max(0.0));
            *normal = rotated.to_array();
            for channel in 0..3 {
                color[channel] *= v.tint[channel] * shade;
            }
        }
    };
    finish(&mut solid);
    let first = mesh.solid.positions.len() as u32;
    mesh.solid.positions.extend(solid.positions);
    mesh.solid
        .cutaway_ranges
        .push(first as usize..mesh.solid.positions.len());
    mesh.solid.normals.extend(solid.normals);
    mesh.solid.uvs.extend(solid.uvs);
    mesh.solid.colors.extend(solid.colors);
    mesh.solid
        .indices
        .extend(solid.indices.into_iter().map(|i| i + first));
    for mut part in parts {
        finish(&mut part.mesh);
        part.mesh.cutaway_ranges = vec![0..part.mesh.positions.len()];
        mesh.scenery.push(part);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn grove_buffers_are_sized_for_every_original_leaf_branch_and_understory() {
        for coordinates in [[0, 0], [2, 4], [-2, -4]] {
            for neighbors in [[false; 4], [true; 4], [true, false, true, false]] {
                let mut mesh = TerrainMeshData::default();
                append_grove_tree(&mut mesh, [0., 16., 0., 16.], 0., 8., coordinates, neighbors);
                let (vertices, indices) = grove_geometry_size(coordinates, neighbors);
                assert_eq!(mesh.solid.positions.len(), vertices);
                assert_eq!(mesh.solid.indices.len(), indices);
                assert_eq!(mesh.solid.positions.capacity(), vertices);
                assert_eq!(mesh.solid.normals.capacity(), vertices);
                assert_eq!(mesh.solid.uvs.capacity(), vertices);
                assert_eq!(mesh.solid.colors.capacity(), vertices);
                assert_eq!(mesh.solid.indices.capacity(), indices);
            }
        }
    }
    #[test]
    fn pointed_blades_keep_coverage_and_outward_fold_normals() {
        for n in [Vec3::Y,Vec3::Z,Vec3::NEG_X,Vec3::new(0.4,0.7,0.2).normalize()] {
            let u=n.cross(if n.y.abs()>0.9 {Vec3::Z} else {Vec3::Y}).normalize();
            let v=n.cross(u);
            let (points,normals)=pointed_leaf(Vec3::ZERO,u,v,n,1.);
            let along: Vec<_>=points.iter().map(|p|p.dot(u)).collect();
            let across: Vec<_>=points.iter().map(|p|p.dot(v)).collect();
            let span=|a:&[f32]| a.iter().copied().fold(f32::NEG_INFINITY,f32::max)
                - a.iter().copied().fold(f32::INFINITY,f32::min);
            assert!(span(&along)>span(&across)*2.3, "leaf blades need pointed, elongated silhouettes");
            assert!(points[0].dot(n)>points[1].dot(n) && points[1].dot(n)>points[5].dot(n));
            let mut area=0.;
            for edge in 0..8 {
                let a=edge+1; let b=(edge+1)%8+1;
                let face=(points[a]-points[0]).cross(points[b]-points[0]);
                area+=face.dot(n)*0.5;
                assert!(face.dot(normals[0]+normals[a]+normals[b])>0.);
            }
            assert!((3.2..3.6).contains(&area), "pointed leaves must retain canopy coverage");
            assert!(normals.iter().all(|n|n.is_finite() && (n.length()-1.).abs()<0.00001));
        }
    }

    #[test]
    fn woodland_normals_follow_lobed_surface_tangents() {
        let radius = Vec3::new(16.0, 9.0, 13.0);
        let mut strongest_bend = 0.0_f32;
        for phase in [0.0, 1.3, 3.7] {
            for row in 1..12 {
                for side in 0..24 {
                    let a = std::f32::consts::PI * row as f32 / 12.0;
                    let b = std::f32::consts::TAU * side as f32 / 24.0;
                    let direction = |a: f32, b: f32| Vec3::new(a.sin()*b.cos(), a.cos(), a.sin()*b.sin());
                    let point = |a: f32, b: f32| direction(a,b)*radius*crown_shape(a,b,phase,true).0;
                    let (lobe, gradient) = crown_shape(a,b,phase,true);
                    let n = crown_normal(direction(a,b),radius,lobe,gradient);
                    let da = point(a+0.001,b)-point(a-0.001,b);
                    let db = point(a,b+0.001)-point(a,b-0.001);
                    assert!(n.dot(db.cross(da).normalize())>0.999, "normal must follow actual crown, not its enclosing oval");
                    assert!((n.length()-1.0).abs()<0.00001 && n.is_finite());
                    strongest_bend = strongest_bend.max(1.0-n.dot((direction(a,b)/radius).normalize()));
                }
            }
        }
        assert!(strongest_bend>0.15, "leaf masses must have distinct surface directions");
    }
    #[test]
    fn route_trunk_scatter_keeps_all_bark_inside_authored_plots() {
        let mut low = Vec2::splat(f32::INFINITY);
        let mut high = Vec2::splat(f32::NEG_INFINITY);
        for x in [-63, -8, -1, 0, 7, 32] {
            for z in [-17, -1, 0, 9, 63] {
                let offset = grove_trunk_offset([x,z],32.0);
                low = low.min(offset);
                high = high.max(offset);
                let mut mesh = TerrainMeshData::default();
                append_grove_tree(&mut mesh,[0.0,32.0,0.0,32.0],3.0,16.0,[x,z],[false;4]);
                assert!(mesh.solid.positions[..119+4*3*77].iter().all(|p|
                    p[0]>0.0 && p[0]<32.0 && p[2]>0.0 && p[2]<32.0 && p[1]>=3.0));
                let mut crown_low=Vec3::splat(f32::INFINITY);
                let mut crown_high=Vec3::splat(f32::NEG_INFINITY);
                for p in &mesh.solid.positions[119+4*3*77..119+4*3*77+13*25] {
                    crown_low=crown_low.min(Vec3::from_array(*p));
                    crown_high=crown_high.max(Vec3::from_array(*p));
                }
                let span=crown_high-crown_low;
                assert!(span.x>span.y*1.15 && span.z>span.y*1.15,
                    "route crowns must spread broadly rather than form upright columns");
            }
        }
        assert!((high-low).min_element()>6.0, "trunk spacing must materially depart from the source grid");
    }

    #[test]
    fn route_groves_keep_trunks_in_source_plots_and_own_their_cutaway_mesh() {
        for neighbors in [[false;4],[true,false,true,false],[true;4]] {
            let mut mesh=TerrainMeshData::default();
            append_grove_tree(&mut mesh,[0.0,32.0,0.0,32.0],3.0,16.0,[4,8],neighbors);
            assert_eq!(mesh.solid.cutaway_ranges,vec![0..mesh.solid.positions.len()]);
            assert!(// All bark, including the four limbs and their eight forks, stays
            // inside the blocked plot; only the canopy may overlap a grove.
            mesh.solid.positions[..119+4*3*77].iter().all(|p|
                p[0]>0.0 && p[0]<32.0 && p[2]>0.0 && p[2]<32.0 && p[1]>=3.0));
            assert!(mesh.solid.positions.iter().all(|p| Vec3::from_array(*p).is_finite()));
            let crown = &mesh.solid.positions[119+4*3*77..119+4*3*77+13*25];
            let nominal = 16.0*((if neighbors.iter().all(|&n| n) {6.4} else {4.8})+variation(4,8).height*0.8);
            let bottom = crown.iter().map(|p| p[1]-3.0).fold(f32::INFINITY,f32::min);
            let top = crown.iter().map(|p| p[1]-3.0).fold(f32::NEG_INFINITY,f32::max);
            assert!(bottom<nominal*0.48, "lower crown must cover the upper bare trunk");
            assert!(bottom>nominal*0.25 && top<nominal*0.95, "retain a readable trunk and mature canopy height");
            for ids in mesh.solid.indices.chunks_exact(3) {
                let [a,b,c]=[ids[0],ids[1],ids[2]].map(|i| Vec3::from_array(mesh.solid.positions[i as usize]));
                let n:Vec3=ids.iter().map(|&i| Vec3::from_array(mesh.solid.normals[i as usize])).sum();
                assert!((b-a).cross(c-a).dot(n)>0.0);
            }
            let mut repeat=TerrainMeshData::default();
            append_grove_tree(&mut repeat,[0.0,32.0,0.0,32.0],3.0,16.0,[4,8],neighbors);
            assert_eq!(mesh.solid.positions,repeat.solid.positions);
            assert_eq!(mesh.solid.colors,repeat.solid.colors);
        }
    }
    #[test]
    fn grove_canopy_thinning_keeps_understory_and_normalized_mesh() {
        let mut skipped=TerrainMeshData::default();
        append_grove_tree(&mut skipped,[0.0,32.0,0.0,32.0],3.0,16.0,[6,8],[true;4]);
        // Canopy thinning must not remove the ground vegetation underneath.
        assert!(!skipped.solid.positions.is_empty());
        assert_eq!(skipped.solid.cutaway_ranges,vec![0..skipped.solid.positions.len()]);
        assert!(skipped.solid.positions.iter().all(|p|
            p[0]>0.0 && p[0]<32.0 && p[2]>0.0 && p[2]<32.0
                && p[1]>=3.0 && p[1]<19.0));
        for neighbors in [[false;4],[true,false,true,false],[true;4]] {
            let mut mesh=TerrainMeshData::default();
            append_grove_tree(&mut mesh,[0.0,32.0,0.0,32.0],3.0,16.0,[4,8],neighbors);
            assert_eq!(mesh.solid.cutaway_ranges,vec![0..mesh.solid.positions.len()]);
            assert!(mesh.solid.positions.iter().any(|p| p[1]==3.0));
            for (p,n) in mesh.solid.positions.iter().zip(&mesh.solid.normals) {
                assert!(Vec3::from_array(*p).is_finite());
                assert!((Vec3::from_array(*n).length()-1.0).abs()<1e-5);
            }
            for ids in mesh.solid.indices.chunks_exact(3) {
                let [a,b,c]=[ids[0],ids[1],ids[2]].map(|i| Vec3::from_array(mesh.solid.positions[i as usize]));
                let n:Vec3=ids.iter().map(|&i| Vec3::from_array(mesh.solid.normals[i as usize])).sum();
                assert!((b-a).cross(c-a).dot(n)>0.0);
            }
            let mut repeat=TerrainMeshData::default();
            append_grove_tree(&mut repeat,[0.0,32.0,0.0,32.0],3.0,16.0,[4,8],neighbors);
            assert_eq!(mesh.solid.positions,repeat.solid.positions);
        }
    }
    #[test]
    fn mature_trees_clear_house_roofs_and_keep_authored_trunk_plots() {
        let house = new_bark_models::model(ModelKind::PlayerHouse);
        let roof = (house.max[1] - house.min[1]) * 16.0;
        for z in 0..20 {
            for x in 0..20 {
                let v = variation(x, z);
                assert!(
                    80.0 * v.height > roof,
                    "mature crown must clear normal house roof"
                );
                assert!(v.offset.iter().all(|o| (o * 16.0 * 0.30).abs() < 8.0));
            }
        }
    }
    #[test]
    fn grove_variation_is_stable_diverse_and_bounded() {
        let mut counts = [0; 5];
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
