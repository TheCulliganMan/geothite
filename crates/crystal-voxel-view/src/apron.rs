//! Thin closed work aprons fitted and skinned to the existing robe surface.
use crate::{
    mesh::SurfaceMeshData,
    npc_appearance::{CostumeMesh, Look},
};
use bevy::prelude::*;

const COLS: usize = 24;
const ROWS: usize = 32;
const GAP: f32 = 0.014;
const THICKNESS: f32 = 0.008;
const BELT_SIDES: usize = 96;
const BELT_ROWS: usize = 2;

#[derive(Clone, Copy)]
struct ApronStyle {
    height: f32,
    normal_gap: f32,
    cloth: [u8; 3],
    pocket: [u8; 3],
    band: [u8; 3],
}
const WORK_APRON: ApronStyle = ApronStyle {
    height: 1.025, normal_gap: 0., cloth: [185, 151, 105], pocket: [169, 135, 92], band: [153, 118, 77],
};
const SHORT_APRON: ApronStyle = ApronStyle {
    height: 0.825, normal_gap: 0.003, cloth: [234, 233, 220], pocket: [222, 221, 204], band: [209, 208, 190],
};

struct Sample {
    z: f32,
    normal: Vec3,
    joints: [u16; 4],
    weights: [f32; 4],
    #[cfg(test)]
    triangle: [usize; 3],
    #[cfg(test)]
    bary: [f32; 3],
}

/// Front ray into the authored cloth; interpolates exactly its skin palette.
fn sample(cloth: &CostumeMesh, x: f32, y: f32) -> Option<Sample> {
    let q = Vec2::new(x, y);
    let cross = |a: Vec2, b: Vec2| a.x * b.y - a.y * b.x;
    let mut result: Option<Sample> = None;
    for face in cloth.surface.indices.chunks_exact(3) {
        let ids = [face[0], face[1], face[2]].map(|i| i as usize);
        let p = ids.map(|i| Vec3::from_array(cloth.surface.positions[i]));
        let a = p[0].truncate();
        let b = p[1].truncate();
        let c = p[2].truncate();
        let area = cross(b - a, c - a);
        if area.abs() < 1e-9 {
            continue;
        }
        let v = cross(q - a, c - a) / area;
        let w = cross(b - a, q - a) / area;
        let bary = [1. - v - w, v, w];
        if bary.iter().any(|&v| v < -1e-5) {
            continue;
        }
        let z = (0..3).map(|i| p[i].z * bary[i]).sum();
        if result.as_ref().is_some_and(|s| s.z >= z) {
            continue;
        }
        let mut skin = std::collections::BTreeMap::<u16, f32>::new();
        for i in 0..3 {
            for j in 0..4 {
                *skin.entry(cloth.joints[ids[i]][j]).or_default() +=
                    bary[i].max(0.) * cloth.weights[ids[i]][j];
            }
        }
        let mut skin = skin.into_iter().collect::<Vec<_>>();
        skin.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        skin.truncate(4);
        let sum = skin.iter().map(|s| s.1).sum::<f32>();
        result = Some(Sample {
            z,
            normal: (p[1] - p[0]).cross(p[2] - p[0]).normalize(),
            #[cfg(test)]
            triangle: ids,
            #[cfg(test)]
            bary,
            joints: std::array::from_fn(|i| skin.get(i).map_or(0, |v| v.0)),
            weights: std::array::from_fn(|i| skin.get(i).map_or(0., |v| v.1 / sum)),
        });
    }
    result
}

/// Outer horizontal section, including the back and sides of the robe.
fn radial_sample(cloth: &CostumeMesh, y: f32, dir: Vec2) -> Option<Sample> {
    let center = Vec2::new(0., 0.05);
    let cross = |a: Vec2, b: Vec2| a.x * b.y - a.y * b.x;
    let mut result: Option<Sample> = None;
    for face in cloth.surface.indices.chunks_exact(3) {
        let ids = [face[0], face[1], face[2]].map(|i| i as usize);
        let p = ids.map(|i| Vec3::from_array(cloth.surface.positions[i]));
        let mut hits = Vec::new();
        for (a, b) in [(0, 1), (1, 2), (2, 0)] {
            if (p[b].y - p[a].y).abs() < 1e-8 {
                continue;
            }
            let t = (y - p[a].y) / (p[b].y - p[a].y);
            if !(0.0..=1.0).contains(&t) {
                continue;
            }
            let q = p[a].lerp(p[b], t);
            let mut bary = [0.; 3];
            bary[a] = 1. - t;
            bary[b] = t;
            hits.push((Vec2::new(q.x, q.z), bary));
        }
        if hits.len() != 2 {
            continue;
        }
        let segment = hits[1].0 - hits[0].0;
        let d = cross(dir, segment);
        if d.abs() < 1e-8 {
            continue;
        }
        let delta = hits[0].0 - center;
        let r = cross(delta, segment) / d;
        let t = cross(delta, dir) / d;
        if r <= 0. || !(-1e-5..=1.00001).contains(&t) || result.as_ref().is_some_and(|s| s.z >= r) {
            continue;
        }
        let t = t.clamp(0., 1.);
        let bary: [f32; 3] = std::array::from_fn(|i| hits[0].1[i] * (1. - t) + hits[1].1[i] * t);
        let mut skin = std::collections::BTreeMap::<u16, f32>::new();
        for i in 0..3 {
            for j in 0..4 {
                *skin.entry(cloth.joints[ids[i]][j]).or_default() +=
                    bary[i] * cloth.weights[ids[i]][j];
            }
        }
        let mut skin = skin.into_iter().collect::<Vec<_>>();
        skin.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        skin.truncate(4);
        let sum = skin.iter().map(|s| s.1).sum::<f32>();
        result = Some(Sample {
            z: r,
            normal: (p[1] - p[0]).cross(p[2] - p[0]).normalize(),
            #[cfg(test)]
            triangle: ids,
            #[cfg(test)]
            bary,
            joints: std::array::from_fn(|i| skin.get(i).map_or(0, |v| v.0)),
            weights: std::array::from_fn(|i| skin.get(i).map_or(0., |v| v.1 / sum)),
        });
    }
    result
}

fn normals(surface: &mut SurfaceMeshData) {
    surface.normals.fill([0.; 3]);
    for f in surface.indices.chunks_exact(3) {
        let p = [f[0], f[1], f[2]].map(|i| Vec3::from_array(surface.positions[i as usize]));
        let n = (p[1] - p[0]).cross(p[2] - p[0]).normalize();
        // Thin rim triangles must influence their corners even when a much
        // larger sloping cloth face meets them. Area weighting can overpower
        // that rim and produce a normal against its outward face.
        for (corner, &i) in f.iter().enumerate() {
            let a = (p[(corner + 1) % 3] - p[corner]).normalize();
            let b = (p[(corner + 2) % 3] - p[corner]).normalize();
            let angle = a.dot(b).clamp(-1., 1.).acos();
            surface.normals[i as usize] =
                (Vec3::from_array(surface.normals[i as usize]) + n * angle).to_array();
        }
    }
    for n in &mut surface.normals {
        *n = Vec3::from_array(*n).normalize().to_array();
    }
}

fn belt(source: &CostumeMesh, style: ApronStyle) -> CostumeMesh {
    let mut surface = SurfaceMeshData::default();
    let mut joints = Vec::new();
    let mut weights = Vec::new();
    let layer = (BELT_ROWS + 1) * BELT_SIDES;
    for inner in [false, true] {
        for row in 0..=BELT_ROWS {
            let y = 1.665 - 0.055 * row as f32 / BELT_ROWS as f32;
            for side in 0..BELT_SIDES {
                let a = std::f32::consts::TAU * side as f32 / BELT_SIDES as f32;
                let dir = Vec2::new(a.cos(), a.sin());
                let fit = radial_sample(source, y, dir).expect("waist belt must fit robe");
                let r = fit.z + 0.018 + if inner { 0. } else { THICKNESS };
                surface.positions.push([dir.x * r, y, 0.05 + dir.y * r]);
                surface.normals.push([0.; 3]);
                surface.uvs.push([0.; 2]);
                surface
                    .colors
                    .push(Color::srgb_u8(style.band[0], style.band[1], style.band[2]).to_linear().to_f32_array());
                joints.push(fit.joints);
                weights.push(fit.weights);
            }
        }
    }
    let at = |r: usize, s: usize| (r * BELT_SIDES + s % BELT_SIDES) as u32;
    let l = layer as u32;
    for r in 0..BELT_ROWS {
        for s in 0..BELT_SIDES {
            let a = at(r, s);
            let b = at(r + 1, s);
            let c = at(r, s + 1);
            let d = at(r + 1, s + 1);
            for f in [[a, c, b], [c, d, b]] {
                surface.indices.extend(f);
                surface.indices.extend([f[2] + l, f[1] + l, f[0] + l]);
            }
        }
    }
    for s in 0..BELT_SIDES {
        let a = at(0, s);
        let b = at(0, s + 1);
        surface.indices.extend([a, a + l, b, b, a + l, b + l]);
        let a = at(BELT_ROWS, s);
        let b = at(BELT_ROWS, s + 1);
        surface.indices.extend([a, b, a + l, b, b + l, a + l]);
    }
    normals(&mut surface);
    CostumeMesh {
        surface,
        joints,
        weights,
    }
}

fn panel(source: &CostumeMesh, pocket: bool, style: ApronStyle) -> CostumeMesh {
    let mut surface = SurfaceMeshData::default();
    let mut joints = Vec::new();
    let mut weights = Vec::new();
    let layer = (COLS + 1) * (ROWS + 1);
    for back in [false, true] {
        for row in 0..=ROWS {
            let t = row as f32 / ROWS as f32;
            let round = ((t - 0.90) / 0.10).max(0.).powi(2);
            let width = if pocket {
                0.155 - 0.015 * round
            } else {
                0.235 + 0.075 * t - 0.025 * round
            };
            for col in 0..=COLS {
                let s = col as f32 / COLS as f32 * 2. - 1.;
                let x = width * s;
                let y = if pocket {
                    1.45 - 0.31 * t + 0.025 * t.powi(6) * s * s
                } else {
                    1.665 - style.height * t + 0.06 * t.powi(6) * s * s
                };
                let fit = sample(source, x, y).expect("apron must fit existing front fabric");
                // Low-amplitude broad folds, faded at waist and hem; no rigid box.
                let fold = if pocket {
                    0.
                } else {
                    0.003 * (s * std::f32::consts::PI * 3.).cos() * (std::f32::consts::PI * t).sin()
                };
                // The raised-leg pose turns the underlying hem almost sideways.
                // Keep a small additional clearance there without lifting the waist.
                let hem_t = ((t - 0.72) / 0.28).clamp(0., 1.);
                let hem_gap = if pocket {
                    0.
                } else {
                    0.014 * hem_t * hem_t * (3. - 2. * hem_t)
                };
                let z = fit.z + GAP + fold + hem_gap + if back { 0. } else { THICKNESS };
                // The shorter skirt tilts steeply during Run. A z-only gap
                // becomes nearly tangent; preserve clearance along its actual
                // outward surface normal without changing the work apron.
                surface.positions.push((Vec3::new(x, y, z) + fit.normal * style.normal_gap).to_array());
                surface.normals.push([0.; 3]);
                let rgb = if pocket {
                    style.pocket
                } else if t < 0.045 {
                    style.band
                } else {
                    style.cloth
                };
                surface.colors.push(
                    Color::srgb_u8(rgb[0], rgb[1], rgb[2])
                        .to_linear()
                        .to_f32_array(),
                );
                surface.uvs.push([0.; 2]);
                joints.push(fit.joints);
                weights.push(fit.weights);
            }
        }
    }
    let at = |r: usize, c: usize| (r * (COLS + 1) + c) as u32;
    for row in 0..ROWS {
        for col in 0..COLS {
            let a = at(row, col);
            let b = at(row + 1, col);
            let c = a + 1;
            let d = b + 1;
            for f in [[a, b, c], [c, b, d]] {
                surface.indices.extend(f);
                surface.indices.extend([
                    f[2] + layer as u32,
                    f[1] + layer as u32,
                    f[0] + layer as u32,
                ]);
            }
        }
    }
    // Ordered perimeter follows the front's outward winding.
    let mut edge = Vec::new();
    for r in 0..=ROWS {
        edge.push(at(r, 0));
    }
    for c in 1..=COLS {
        edge.push(at(ROWS, c));
    }
    for r in (0..ROWS).rev() {
        edge.push(at(r, COLS));
    }
    for c in (1..COLS).rev() {
        edge.push(at(0, c));
    }
    for i in 0..edge.len() {
        let a = edge[i];
        let b = edge[(i + 1) % edge.len()];
        let l = layer as u32;
        surface.indices.extend([a, a + l, b, b, a + l, b + l]);
    }
    normals(&mut surface);
    CostumeMesh {
        surface,
        joints,
        weights,
    }
}

fn merge(target: &mut CostumeMesh, extra: CostumeMesh) {
    let start = target.surface.positions.len() as u32;
    target.surface.positions.extend(extra.surface.positions);
    target.surface.normals.extend(extra.surface.normals);
    target.surface.colors.extend(extra.surface.colors);
    target.surface.uvs.extend(extra.surface.uvs);
    target
        .surface
        .indices
        .extend(extra.surface.indices.into_iter().map(|i| i + start));
    target.joints.extend(extra.joints);
    target.weights.extend(extra.weights);
}

pub(crate) fn append(cloth: &mut CostumeMesh, look: Look) {
    let style = match look.texture_key {
        "kurt" | "kurt_outside" => WORK_APRON,
        "pokefan_f" | "mom" | "reds_mom" => SHORT_APRON,
        _ => return,
    };
    let mut apron = panel(cloth, false, style);
    let waist_belt = belt(cloth, style);
    let pocket = panel(&apron, true, style);
    merge(&mut apron, pocket);
    merge(cloth, apron);
    merge(cloth, waist_belt);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{new_bark_actors::character_meshes::CharacterKind, open_people::PersonRig};
    fn world(i: usize, pose: &[Transform], parents: &[Option<usize>]) -> Mat4 {
        parents[i]
            .map(|p| world(p, pose, parents))
            .unwrap_or(Mat4::IDENTITY)
            * pose[i].to_matrix()
    }
    #[test]
    #[ignore = "requires supplied male/female rigs and paintings in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_aprons_are_closed_fitted_and_follow_the_robe() {
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let male =
            PersonRig::parse(&std::fs::read(root.join("people/kenney/human-male.glb")).unwrap())
                .unwrap();
        let female =
            PersonRig::parse(&std::fs::read(root.join("people/kenney/human-female.glb")).unwrap())
                .unwrap();
        for kind in [CharacterKind::Kurt, CharacterKind::KurtOutside,
            CharacterKind::PokefanF, CharacterKind::Mom, CharacterKind::RedsMom] {
            let look = crate::npc_appearance::look(kind);
            let short = matches!(kind, CharacterKind::PokefanF | CharacterKind::Mom | CharacterKind::RedsMom);
            let rig = if short { &female } else { &male };
            let style = if short { SHORT_APRON } else { WORK_APRON };
            let costume = crate::npc_appearance::costume_mesh(&rig, look).unwrap();
            let count = 2 * (COLS + 1) * (ROWS + 1);
            let apron_start =
                costume.surface.positions.len() - count * 2 - 2 * (BELT_ROWS + 1) * BELT_SIDES;
            let source = CostumeMesh {
                surface: SurfaceMeshData {
                    positions: costume.surface.positions[..apron_start].to_vec(),
                    normals: costume.surface.normals[..apron_start].to_vec(),
                    colors: costume.surface.colors[..apron_start].to_vec(),
                    uvs: costume.surface.uvs[..apron_start].to_vec(),
                    indices: costume
                        .surface
                        .indices
                        .iter()
                        .copied()
                        .take_while(|&i| (i as usize) < apron_start)
                        .collect(),
                    ..Default::default()
                },
                joints: costume.joints[..apron_start].to_vec(),
                weights: costume.weights[..apron_start].to_vec(),
            };
            let apron = panel(&source, false, style);
            let pocket = panel(&apron, true, style);
            let waist_belt = belt(&source, style);
            for (piece_index, piece) in [&apron, &pocket, &waist_belt].into_iter().enumerate() {
                let mut edges = std::collections::BTreeMap::<(u32, u32), (u32, i32)>::new();
                for f in piece.surface.indices.chunks_exact(3) {
                    let p = [f[0], f[1], f[2]]
                        .map(|i| Vec3::from_array(piece.surface.positions[i as usize]));
                    assert!((p[1] - p[0]).cross(p[2] - p[0]).length_squared() > 1e-16);
                    let n = f
                        .iter()
                        .map(|&i| Vec3::from_array(piece.surface.normals[i as usize]))
                        .sum::<Vec3>();
                    assert!(
                        (p[1] - p[0]).cross(p[2] - p[0]).dot(n) > 0.,
                        "inward apron normals piece {piece_index} face {f:?} points {p:?} normal sum {n:?}"
                    );
                    for (a, b) in [(f[0], f[1]), (f[1], f[2]), (f[2], f[0])] {
                        let e = edges.entry((a.min(b), a.max(b))).or_default();
                        e.0 += 1;
                        e.1 += if a < b { 1 } else { -1 };
                    }
                }
                assert!(
                    edges.values().all(|&v| v == (2, 0)),
                    "open or inverted apron"
                );
                for (n, w) in piece.surface.normals.iter().zip(&piece.weights) {
                    assert!((Vec3::from_array(*n).length() - 1.).abs() < 1e-5);
                    assert!(w.iter().all(|w| w.is_finite() && *w >= 0.));
                    assert!((w.iter().sum::<f32>() - 1.).abs() < 1e-5);
                }
            }
            let fits = [(&apron, &source), (&pocket, &apron)].map(|(piece, under)| {
                piece
                    .surface
                    .positions
                    .iter()
                    .map(|p| sample(under, p[0], p[1]).unwrap())
                    .collect::<Vec<_>>()
            });
            let belt_fits = waist_belt
                .surface
                .positions
                .iter()
                .map(|p| {
                    let dir = Vec2::new(p[0], p[2] - 0.05).normalize();
                    let fit = radial_sample(&source, p[1], dir).unwrap();
                    (Vec3::new(dir.x * fit.z, p[1], 0.05 + dir.y * fit.z), fit)
                })
                .collect::<Vec<_>>();
            let mut closest = (f32::INFINITY, String::new());
            let mut furthest = 0.0_f32;
            for clip in ["Idle", "Run"] {
                for step in 0..=60 {
                    let mut pose = rig.nodes.clone();
                    rig.sample(clip, rig.duration(clip) * step as f32 / 60., &mut pose);
                    let inverse = world(rig.mesh_node, &pose, &rig.parents).inverse();
                    let matrices = rig
                        .skin_nodes
                        .iter()
                        .zip(&rig.inverse_binds)
                        .map(|(&n, b)| inverse * world(n, &pose, &rig.parents) * *b)
                        .collect::<Vec<_>>();
                    let deform = |p: Vec3, ids: [u16; 4], w: [f32; 4]| {
                        (0..4)
                            .map(|j| matrices[ids[j] as usize].transform_point3(p) * w[j])
                            .sum::<Vec3>()
                    };
                    let under_surface = |cloth: &CostumeMesh, fit: &Sample| {
                        let p = fit.triangle.map(|i| {
                            deform(
                                Vec3::from_array(cloth.surface.positions[i]),
                                cloth.joints[i],
                                cloth.weights[i],
                            )
                        });
                        let point = (0..3).map(|i| p[i] * fit.bary[i]).sum::<Vec3>();
                        (point, (p[1] - p[0]).cross(p[2] - p[0]).normalize())
                    };
                    for (piece_index, ((piece, under), fits)) in
                        [(&apron, &source), (&pocket, &apron)]
                            .into_iter()
                            .zip(&fits)
                            .enumerate()
                    {
                        for (i, fit) in fits.iter().enumerate() {
                            let p = Vec3::from_array(piece.surface.positions[i]);
                            let (point, normal) = under_surface(under, fit);
                            let gap = deform(p, piece.joints[i], piece.weights[i]) - point;
                            let clearance = gap.dot(normal);
                            furthest = furthest.max(gap.length());
                            if clearance < closest.0 {
                                closest = (
                                    clearance,
                                    format!(
                                        "{kind:?} {clip} step {step} piece {piece_index} vertex {i} bind {p:?} normal {normal:?} gap {gap:?}"
                                    ),
                                );
                            }
                            let animated = deform(p, piece.joints[i], piece.weights[i]);
                            assert!(
                                animated.cmpge(rig.bounds.0 - Vec3::splat(0.6)).all()
                                    && animated.cmple(rig.bounds.1 + Vec3::splat(0.6)).all()
                            );
                        }
                    }
                    for (i, (_, fit)) in belt_fits.iter().enumerate() {
                        let p = Vec3::from_array(waist_belt.surface.positions[i]);
                        let animated = deform(p, waist_belt.joints[i], waist_belt.weights[i]);
                        let (point, normal) = under_surface(&source, fit);
                        let gap = animated - point;
                        let clearance = gap.dot(normal);
                        furthest = furthest.max(gap.length());
                        if clearance < closest.0 {
                            closest = (
                                clearance,
                                format!(
                                    "{kind:?} {clip} step {step} belt vertex {i} bind {p:?} normal {normal:?} gap {gap:?}"
                                ),
                            );
                        }
                        assert!(
                            animated.cmpge(rig.bounds.0 - Vec3::splat(0.6)).all()
                                && animated.cmple(rig.bounds.1 + Vec3::splat(0.6)).all()
                        );
                    }
                }
            }
            assert!(
                closest.0 > 0.004 && furthest < 0.045,
                "minimum {} maximum {furthest}: {}",
                closest.0,
                closest.1
            );
            assert_eq!(
                costume.surface.positions[apron_start..apron_start + count],
                apron.surface.positions
            );
        }
        let look = crate::npc_appearance::look(CharacterKind::Elder);
        let mut costume = crate::npc_appearance::costume_mesh(&male, look).unwrap();
        let before = costume.surface.positions.len();
        append(&mut costume, look);
        assert_eq!(costume.surface.positions.len(), before);
    }
}
