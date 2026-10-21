//! Shared, model-local craft detail for authored scenery. Source pixels,
//! textured foliage, actor skins, placement and support remain authoritative.
use crate::mesh::SurfaceMeshData;
use bevy::prelude::Vec3;

// Live source paint already contains the current map's time-of-day palette.
// Retain shadow/contact modulation without applying a second colored light.
pub(crate) const SOURCE_PAINT_UV: [f32;2] = [85000.,0.];
// Native dry-stone pigment with the same world-anchored mineral detail as sea stones.
pub(crate) const SOURCE_STONE_UV: [f32;2] = [85000.,85000.];
// Raised brick geometry has already been fitted to its pier's source-corner
// heights. Retain the same live-paint shader role while giving terrain passes
// explicit ownership at clipped bevels exactly on a land/deck boundary.
pub(crate) const LEVEL_PAVING_UV: [f32;2] = [85000.,-61000.];
// Live source pigment uses the same woven finish as static entry cloth.
pub(crate) const SOURCE_FABRIC_BASE: [f32;2] = [82000.,3500.];
// The same mat turned across a side gate has a 16x32 rather than 32x16
// source footprint. Keep its perimeter binding continuous across all cells.
pub(crate) const SOURCE_FABRIC_SIDE_BASE: [f32;2] = [82000.,3700.];
pub(crate) fn source_fabric_coordinates(uv:[f32;2])->Option<[f32;2]> {
    let p=[uv[0]-SOURCE_FABRIC_BASE[0],uv[1]-SOURCE_FABRIC_BASE[1]];
    (p.iter().all(|v|v.is_finite()) && (0. ..100.).contains(&p[0])
        && (0. ..500.).contains(&p[1])).then_some(p)
}
// City paving carries source-map coordinates for stationary mineral grain.
pub(crate) const PAVING_GRAIN_BASE: [f32;2] = [83000.,-3000.];
pub(crate) const ROAD_GRAIN_BASE: [f32;2] = [80000.,-3000.];
pub(crate) fn road_coordinates(uv: [f32;2]) -> Option<[f32;2]> {
    (uv.iter().all(|v|v.is_finite()) && (79500. ..81500.).contains(&uv[0])
        && (-5000. ..-2000.).contains(&uv[1])).then(||
            [uv[0]-ROAD_GRAIN_BASE[0],ROAD_GRAIN_BASE[1]-uv[1]])
}
pub(crate) fn paving_coordinates(uv: [f32;2]) -> Option<[f32;2]> {
    if let Some(p)=road_coordinates(uv) {return Some(p);}
    (uv.iter().all(|v|v.is_finite()) && (82500. ..84000.).contains(&uv[0])
        && (-5000. ..-2000.).contains(&uv[1])).then(||
            [uv[0]-PAVING_GRAIN_BASE[0],PAVING_GRAIN_BASE[1]-uv[1]])
}
// Live atlas paint already carries time-of-day color. A reversible
// negative U band marks that lighting policy without copying source pixels.
pub(crate) fn source_atlas_uv(uv: [f32;2]) -> [f32;2] {[-2.-uv[0],uv[1]]}
// Sculpted floats keep the live palette and receive their own face response.
pub(crate) fn source_buoy_uv(uv: [f32;2]) -> [f32;2] {[-8.-uv[0],uv[1]]}
pub(crate) fn source_buoy_coordinates(uv: [f32;2]) -> Option<[f32;2]> {
    (uv.iter().all(|v|v.is_finite()) && (-9. ..=-8.).contains(&uv[0])
        && (0. ..=1.).contains(&uv[1])).then(||[-8.-uv[0],uv[1]])
}
// Concrete samples live native pigment without magnifying its pixel speckles.
pub(crate) fn source_concrete_uv(uv:[f32;2])->[f32;2] {[-6.-uv[0],uv[1]]}
pub(crate) fn source_concrete_coordinates(uv:[f32;2])->Option<[f32;2]> {
    (uv.iter().all(|v|v.is_finite()) && (-7. ..=-6.).contains(&uv[0])
        && (0. ..=1.).contains(&uv[1])).then(||[-6.-uv[0],uv[1]])
}
// Mineral source paint retains its live value while softening palette chroma.
pub(crate) fn source_mineral_uv(uv: [f32;2]) -> [f32;2] {[-4.-uv[0],uv[1]]}
pub(crate) fn source_mineral_coordinates(uv: [f32;2]) -> Option<[f32;2]> {
    (uv.iter().all(|v|v.is_finite()) && (-5. ..=-4.).contains(&uv[0])
        && (0. ..=1.).contains(&uv[1])).then(||[-4.-uv[0],uv[1]])
}
pub(crate) fn source_atlas_coordinates(uv: [f32;2]) -> Option<[f32;2]> {
    if let Some(p)=source_buoy_coordinates(uv) {return Some(p);}
    (uv.iter().all(|v|v.is_finite()) && (-3. ..=-2.).contains(&uv[0])
        && (0. ..=1.).contains(&uv[1])).then(||[-2.-uv[0],uv[1]])
}

// Four town paint domains carry authored local coordinates in the same UV
// channel: ceramic, plaster, clay and mortar. Existing source paint is separate.
pub(crate) const TOWN_PAINT_BASE: f32 = 1024.;
pub(crate) const TOWN_GLASS_BASE: f32 = 1152.;
pub(crate) const TOWN_TIMBER_BASE: f32 = 1184.;
// Mineral slate is separate from ceramic glaze, glass and source panes.
pub(crate) const TOWN_SLATE_BASE: f32 = 1328.;
// Closed source-colored panes retain normalized coordinates independently of
// whether their drawing is an authored lamp. Two disjoint bands keep dark
// glass out of the nighttime emitter list.
pub(crate) const SOURCE_GLASS_BASE: f32 = 1216.;
/// Tag the closed pane's local XY extent independently of its native lamp
/// identity. Both compact office bays and continuous museum bands share it.
pub(crate) fn mark_source_glass(mesh: &mut SurfaceMeshData,
    vertices: std::ops::Range<usize>, bounds: [f32;4], lit: bool) {
    let [left,right,bottom,top]=bounds;
    for i in vertices {
        let p=mesh.positions[i];
        mesh.uvs[i]=[((p[0]-left)/(right-left)).clamp(0.,1.),
            SOURCE_GLASS_BASE+if lit {2.}else{0.}
                +((p[1]-bottom)/(top-bottom)).clamp(0.,1.)];
    }
}
pub(crate) fn source_glass_coordinates(uv: [f32; 2]) -> Option<([f32; 2], bool)> {
    for lit in [false, true] {
        let p = [uv[0], uv[1] - SOURCE_GLASS_BASE - if lit { 2. } else { 0. }];
        if p.iter().all(|v| v.is_finite() && (0. ..=1.).contains(v)) {
            return Some((p, lit));
        }
    }
    None
}

pub(crate) fn town_timber_coordinates(uv: [f32;2]) -> Option<[f32;2]> {
    let p=[uv[0],uv[1]-TOWN_TIMBER_BASE];
    p.iter().all(|v|v.is_finite() && (0. ..16.).contains(v)).then_some(p)
}
pub(crate) fn town_glass_coordinates(uv: [f32; 2]) -> Option<[f32; 2]> {
    let p = [uv[0], uv[1] - TOWN_GLASS_BASE];
    p.iter().all(|v| v.is_finite() && (0. ..=1.).contains(v)).then_some(p)
}
// Town cast shade needs enough separation to ground foundations and actors.
// Keep this independent of the wider exterior/forest daylight policy.
pub(crate) fn town_shadow_fill(map: &str, tileset: &str) -> bool {
    (map.ends_with("City") || map.ends_with("Town"))
        && matches!(tileset, "johto" | "johto_modern" | "kanto")
}
pub(crate) fn town_paint_kind(uv: [f32;2]) -> Option<u8> {
    if uv.iter().all(|v|v.is_finite()) && uv[0].abs()<16.
        && (uv[1]-TOWN_SLATE_BASE).abs()<8. {return Some(4);}
    let kind=((uv[1]-TOWN_PAINT_BASE)/32.).round();
    (uv.iter().all(|v|v.is_finite()) && (0.0..=3.0).contains(&kind)
        && uv[0].abs()<16. && (uv[1]-(TOWN_PAINT_BASE+kind*32.)).abs()<8.).then_some(kind as u8)
}

#[derive(Clone, Copy)]
enum Material {
    Ceramic,
    Timber,
    Plaster,
    Stone,
    Foliage,
}
fn material(name: &str) -> Option<Material> {
    if name.starts_with("roof_") && !name.ends_with("_dark") {
        Some(Material::Ceramic)
    } else if name.contains("timber") || name.contains("wood") || name.contains("oak") {
        Some(Material::Timber)
    } else if name.starts_with("plaster") {
        Some(Material::Plaster)
    } else if name.starts_with("leaf") {
        Some(Material::Foliage)
    } else if name.starts_with("stone") {
        Some(Material::Stone)
    } else {
        None
    }
}
fn noise(p: Vec3) -> f32 {
    // Smooth, bounded variation in authored model coordinates. Scrolling,
    // camera orbit and repeated placements cannot move the painted grain.
    (p.x * 11.7 + (p.z * 7.3).sin()).sin() * (p.y * 8.9 + (p.x * 3.1).cos()).cos()
}
pub(crate) fn timber_grain(p: [f32; 3]) -> f32 {
    let p = Vec3::from_array(p);
    noise(Vec3::new(p.x * 2.8, p.y * 0.10, p.z * 2.8))
}
fn paint(base: [f32; 4], p: Vec3, role: Material) -> [f32; 4] {
    let grain = match role {
        Material::Ceramic => noise(p * 0.8) * 0.024,
        Material::Timber => timber_grain(p.to_array()) * 0.038,
        Material::Plaster => noise(p * 0.55) * 0.017,
        Material::Stone => noise(p * 1.5) * 0.025,
        Material::Foliage => noise(p * 1.7) * 0.045,
    };
    [
        (base[0] + grain).clamp(0., 1.),
        (base[1] + grain * 0.86).clamp(0., 1.),
        (base[2] + grain * 0.65).clamp(0., 1.),
        base[3],
    ]
}
fn triangle(
    out: &mut SurfaceMeshData,
    p: [Vec3; 3],
    n: [Vec3; 3],
    uv: [[f32; 2]; 3],
    base: [f32; 4],
    role: Material,
) {
    let start = out.positions.len() as u32;
    out.positions.extend(p.map(|v| v.to_array()));
    out.normals.extend(n.map(|v| v.normalize().to_array()));
    out.uvs.extend(uv);
    out.colors.extend(p.map(|v| paint(base, v, role)));
    out.indices.extend([start, start + 1, start + 2]);
}
pub(crate) fn contain(mesh: &mut SurfaceMeshData, min: [f32; 3], max: [f32; 3]) {
    for p in &mut mesh.positions {
        for axis in 0..3 {
            p[axis] = p[axis].clamp(min[axis], max[axis]);
        }
    }
}
#[cfg(test)]
fn face(out: &mut SurfaceMeshData, p: [Vec3; 3], base: [f32; 4], role: Material) {
    let normal = (p[1] - p[0]).cross(p[2] - p[0]).normalize();
    triangle(out, p, [normal; 3], [[0.; 2]; 3], base, role);
}
fn roof_quad(old: &SurfaceMeshData, ids: &[u32]) -> Option<[Vec3; 4]> {
    if ids.len() != 6 {
        return None;
    }
    let normal = Vec3::from_array(old.normals[ids[0] as usize]);
    if normal.y < 0.35 || normal.y > 0.98 {
        return None;
    }
    if ids
        .iter()
        .any(|i| Vec3::from_array(old.normals[*i as usize]).dot(normal) < 0.9999)
    {
        return None;
    }
    let mut boundary = Vec::new();
    for tri in ids.chunks_exact(3) {
        for (a, b) in [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])] {
            let a = Vec3::from_array(old.positions[a as usize]);
            let b = Vec3::from_array(old.positions[b as usize]);
            if let Some(i) = boundary.iter().position(|&(x, y)| x == b && y == a) {
                boundary.remove(i);
            } else {
                boundary.push((a, b));
            }
        }
    }
    if boundary.len() != 4 {
        return None;
    }
    let mut cursor = boundary[0].0;
    let mut points = Vec::new();
    for _ in 0..4 {
        points.push(cursor);
        cursor = boundary.iter().find(|(a, _)| *a == cursor)?.1;
    }
    let p: [Vec3; 4] = points.try_into().ok()?;
    let area = (p[1] - p[0]).cross(p[3] - p[0]).length();
    // Small trim and broad structural roof decking retain their geometry.
    if !(0.025..0.8).contains(&area) || p.iter().any(|v| (*v - p[0]).dot(normal).abs() > 0.0001) {
        return None;
    }
    Some(p)
}
fn leaves(out: &mut SurfaceMeshData, p: [Vec3; 3], base: [f32; 4]) {
    let normal = (p[1] - p[0]).cross(p[2] - p[0]).normalize();
    let center = (p[0] + p[1] + p[2]) / 3.;
    // Three-leaf sprays follow the real canopy surface. No spheres,
    // source occupancy expansion or camera-facing billboard is substituted.
    let seed = (center.dot(Vec3::new(13.31, 7.79, 17.17)).sin() * 437.37)
        .fract()
        .abs();
    if normal.y < -0.35 || seed < 0.35 {
        return;
    }
    let radius = (p[1] - p[0])
        .length()
        .min((p[2] - p[1]).length())
        .min((p[0] - p[2]).length());
    let length = (radius * 0.52).clamp(0.045, 0.22);
    let root = center + normal * 0.009;
    let tangent = normal
        .cross(if normal.y.abs() > 0.95 {
            Vec3::X
        } else {
            Vec3::Y
        })
        .normalize();
    let bitangent = normal.cross(tangent);
    for leaf in 0..3 {
        let angle = seed * 6.283185 + leaf as f32 * 2.094395;
        let forward = tangent * angle.cos() + bitangent * angle.sin();
        let side = normal.cross(forward) * length * 0.28;
        let mid = root + forward * length * 0.54;
        let ridge = mid + normal * length * 0.24;
        let color = [
            (base[0] + 0.045 + seed * 0.055).min(1.),
            (base[1] + 0.065 + seed * 0.045).min(1.),
            (base[2] + seed * 0.009).min(1.),
            base[3],
        ];
        // Rounded leaf margins and a raised central vein give the canopy
        // individual leaf silhouettes instead of diamond-shaped studs.
        let outline: [Vec3; 8] = std::array::from_fn(|i| {
            let a = i as f32 * std::f32::consts::TAU / 8.;
            mid + forward * (a.cos() * length * 0.50) + side * a.sin() + normal * length * 0.06
        });
        for i in 0..8 {
            let mut points = [ridge, outline[i], outline[(i + 1) % 8]];
            if (points[1] - points[0])
                .cross(points[2] - points[0])
                .dot(normal)
                < 0.
            {
                points.swap(1, 2);
            }
            let normals =
                points.map(|p| (normal + (p - mid).normalize_or_zero() * 0.30).normalize());
            triangle(out, points, normals, [[0.; 2]; 3], color, Material::Foliage);
        }
    }
}
/// Refine only the just-imported primitive; unknown or textured materials are
/// never inferred from color. The rest of the model stays byte-for-byte intact.
pub(crate) fn refine(
    mesh: &mut SurfaceMeshData,
    vertex_start: usize,
    index_start: usize,
    name: &str,
    object: &str,
) {
    if !mesh.cutaway_ranges.is_empty() {
        return;
    }
    let Some(role) = material(name) else {
        return;
    };
    let old = SurfaceMeshData {
        positions: mesh.positions.split_off(vertex_start),
        normals: mesh.normals.split_off(vertex_start),
        uvs: mesh.uvs.split_off(vertex_start),
        colors: mesh.colors.split_off(vertex_start),
        indices: mesh
            .indices
            .split_off(index_start)
            .into_iter()
            .map(|i| i - vertex_start as u32)
            .collect(),
        ..Default::default()
    };
    let mut old = old;
    if matches!(role, Material::Ceramic) {
        let mut moves = std::collections::HashMap::<[u32; 3], Vec3>::new();
        for ids in old.indices.chunks_exact(6) {
            let Some(p) = roof_quad(&old, ids) else {
                continue;
            };
            // Only closed thin panels have corresponding lower corners.
            // Open paint cards and the structural deck retain their plane.
            if !p.iter().all(|top| {
                old.positions.iter().any(|bottom| {
                    (bottom[0] - top.x).abs() < 0.00001
                        && (bottom[2] - top.z).abs() < 0.00001
                        && (0.01..0.08).contains(&(top.y - bottom[1]))
                })
            }) {
                continue;
            }
            let normal = (p[1] - p[0]).cross(p[2] - p[0]).normalize();
            let center = p.iter().copied().sum::<Vec3>() * 0.25;
            for top in p {
                moves.insert(
                    top.to_array().map(f32::to_bits),
                    center + (top - center) * 0.94 + normal * 0.009,
                );
            }
        }
        for p in &mut old.positions {
            if let Some(new) = moves.get(&p.map(f32::to_bits)) {
                *p = new.to_array();
            }
        }
    }
    if matches!(role, Material::Ceramic) {
        for ids in old.indices.chunks_exact(3) {
            let ids = [ids[0] as usize, ids[1] as usize, ids[2] as usize];
            let p = ids.map(|j| Vec3::from_array(old.positions[j]));
            let cross = (p[1] - p[0]).cross(p[2] - p[0]);
            if cross.length_squared() > 0.00000001 {
                for j in ids {
                    old.normals[j] = cross.normalize().to_array();
                }
            }
        }
    }
    for (i, p) in old.positions.iter().enumerate() {
        old.colors[i] = paint(old.colors[i], Vec3::from_array(*p), role);
    }
    let base = mesh.positions.len() as u32;
    mesh.positions.extend_from_slice(&old.positions);
    mesh.normals.extend_from_slice(&old.normals);
    mesh.uvs.extend_from_slice(&old.uvs);
    mesh.colors.extend_from_slice(&old.colors);
    mesh.indices.extend(old.indices.iter().map(|i| base + i));
    let canopy = !object.contains("lod")
        && (object.contains("tree") || object.contains("canopy") || object.contains("conifer"));
    if canopy && matches!(role, Material::Foliage) {
        for ids in old.indices.chunks_exact(3) {
            let ids = [ids[0] as usize, ids[1] as usize, ids[2] as usize];
            leaves(
                mesh,
                ids.map(|j| Vec3::from_array(old.positions[j])),
                old.colors[ids[0]],
            );
        }
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn float_paint_roundtrips_live_coordinates_without_selecting_another_material() {
        for u in [0.,0.03125,0.5,0.96875,1.] {for v in [0.,0.5,1.] {
            let uv=super::source_buoy_uv([u,v]);
            assert_eq!(super::source_buoy_coordinates(uv),Some([u,v]));
            assert_eq!(super::source_atlas_coordinates(uv),Some([u,v]));
            assert!(super::source_concrete_coordinates(uv).is_none());
            assert!(super::source_mineral_coordinates(uv).is_none());
            assert!(super::town_paint_kind(uv).is_none());
            for other in [super::source_atlas_uv([u,v]),super::source_concrete_uv([u,v]),
                super::source_mineral_uv([u,v]),[u,v]] {
                assert!(super::source_buoy_coordinates(other).is_none());
            }
        }}
        for uv in [[f32::NAN,0.],[f32::INFINITY,0.],[-8.5,-0.01],[-8.5,1.01],[-9.01,0.],[-7.99,0.]] {
            assert!(super::source_buoy_coordinates(uv).is_none());
        }
    }

    #[test]
    fn slate_material_domain_is_disjoint_from_glass_and_existing_town_paint() {
        for x in [-7.,0.,7.] {for y in [-7.,0.,7.] {
            let uv=[x,TOWN_SLATE_BASE+y];
            assert_eq!(town_paint_kind(uv),Some(4));
            assert!(source_glass_coordinates(uv).is_none());
            assert!(town_glass_coordinates(uv).is_none());
            assert!(!(crate::city_lights::UV_BASE..=crate::city_lights::UV_BASE+3.).contains(&uv[1]),
                "slate cannot overlap any legacy window, cream or neon emitter role");
            assert!(!crate::city_lights::source_window_tag(uv));
        }}
        for kind in 0..4 {
            assert_eq!(town_paint_kind([0.,TOWN_PAINT_BASE+kind as f32*32.]),Some(kind));
        }
        for base in [TOWN_GLASS_BASE,TOWN_TIMBER_BASE,SOURCE_GLASS_BASE,
            crate::city_lights::UV_BASE,crate::city_lights::SOURCE_UV_BASE] {
            for y in [0.,0.25,1.] {assert_eq!(town_paint_kind([0.,base+y]),None);}
        }
    }
    use super::*;
    #[test]
    fn closed_ceramic_panel_gains_a_crown_without_more_triangles() {
        let top = [
            Vec3::new(0., 1., 0.),
            Vec3::new(0., 1.25, 0.5),
            Vec3::new(0.5, 1.25, 0.5),
            Vec3::new(0.5, 1., 0.),
        ];
        let bottom = top.map(|v| v - Vec3::Y * 0.04);
        let mut m = SurfaceMeshData::default();
        for q in [
            top,
            [bottom[3], bottom[2], bottom[1], bottom[0]],
            [bottom[0], bottom[1], top[1], top[0]],
            [bottom[1], bottom[2], top[2], top[1]],
            [bottom[2], bottom[3], top[3], top[2]],
            [bottom[3], bottom[0], top[0], top[3]],
        ] {
            for ids in [[0, 1, 2], [0, 2, 3]] {
                face(
                    &mut m,
                    ids.map(|i| q[i]),
                    [0.6, 0.3, 0.2, 1.],
                    Material::Ceramic,
                );
            }
        }
        let budget = m.indices.len();
        refine(&mut m, 0, 0, "roof_red", "test_roof");
        assert_eq!(m.indices.len(), budget);
        for v in bottom {
            assert!(m.positions.contains(&v.to_array()));
        }
        assert!(!m.positions.contains(&top[0].to_array()));
        for n in &m.normals {
            assert!((Vec3::from_array(*n).length() - 1.).abs() < 0.001);
        }
        assert!(m.colors.iter().all(|c| c[3] == 1.));
    }
}
