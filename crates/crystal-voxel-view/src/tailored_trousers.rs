//! Joined trouser panels with a real crotch, two leg openings and broad hems.
//! The cut follows the supplied leg axes without copying the body's small folds.
use crate::{
    mesh::SurfaceMeshData,
    npc_appearance::{CostumeMesh, Look},
    open_people::PersonRig,
};
use bevy::prelude::*;
use std::collections::BTreeMap;

const ROWS: usize = 16;

fn smooth(value: f32) -> f32 {
    let t = value.clamp(0., 1.);
    t * t * (3. - 2. * t)
}

fn section_skin(
    body: &crate::open_people::RefinedBody,
    faces: &[[usize; 3]],
    y: f32,
    side: usize,
) -> Vec<(u16, f32)> {
    let y = y.clamp(0.259, 1.46);
    let mut merged = BTreeMap::<u16, f32>::new();
    let mut total = 0.;
    for face in faces {
        let p = face.map(|i| Vec3::from_array(body.mesh.positions[i]));
        let sign = p.iter().map(|p| p.x).sum::<f32>();
        if (side == 0 && sign < 0.) || (side == 1 && sign > 0.) {
            continue;
        }
        let mut hits = Vec::new();
        for (a, b) in [(0, 1), (1, 2), (2, 0)] {
            if (p[a].y < y) == (p[b].y < y) {
                continue;
            }
            let t = (y - p[a].y) / (p[b].y - p[a].y);
            hits.push((p[a].lerp(p[b], t), a, b, t));
        }
        if hits.len() != 2 {
            continue;
        }
        let length = hits[0].0.distance(hits[1].0);
        total += length;
        for &(_, a, b, t) in &hits {
            for (corner, weight) in [(a, (1. - t) * length * 0.5), (b, t * length * 0.5)] {
                for slot in 0..4 {
                    let i = face[corner];
                    *merged.entry(body.ids[i][slot]).or_default() += body.weights[i][slot] * weight;
                }
            }
        }
    }
    assert!(total > 0., "missing supplied trouser section at {y}");
    merged
        .into_iter()
        .filter(|(_, w)| *w > 0.)
        .map(|(j, w)| (j, w / total))
        .collect()
}

/// Fit a horizontal ray to the retained body. A side limit samples one leg;
/// the connected hip boundary samples both halves in the same frame.
pub(crate) fn section_radius(
    positions: &[[f32; 3]],
    faces: &[[usize; 3]],
    y: f32,
    center: Vec2,
    direction: Vec2,
    side: Option<usize>,
) -> Option<f32> {
    let cross = |a: Vec2, b: Vec2| a.x * b.y - a.y * b.x;
    let mut best = 0_f32;
    for face in faces {
        let p = face.map(|i| Vec3::from_array(positions[i]));
        let sign = p.iter().map(|p| p.x).sum::<f32>();
        if (side == Some(0) && sign < 0.) || (side == Some(1) && sign > 0.) {
            continue;
        }
        let mut hits = Vec::new();
        for (a, b) in [(0, 1), (1, 2), (2, 0)] {
            if (p[a].y < y) == (p[b].y < y) {
                continue;
            }
            let t = (y - p[a].y) / (p[b].y - p[a].y);
            let point = p[a].lerp(p[b], t);
            hits.push(Vec2::new(point.x, point.z));
        }
        if hits.len() != 2 {
            continue;
        }
        let segment = hits[1] - hits[0];
        let determinant = cross(direction, segment);
        if determinant.abs() < 1e-8 {
            continue;
        }
        let offset = hits[0] - center;
        let radius = cross(offset, segment) / determinant;
        let t = cross(offset, direction) / determinant;
        if radius > best && (-1e-5..=1.00001).contains(&t) {
            best = radius;
        }
    }
    (best > 0.).then_some(best)
}

/// Blend the connected hip/crotch panels over their surface graph. A spatial
/// left/right test is discontinuous at the welded saddle: a near-seam vertex
/// can otherwise follow one thigh while its neighbor follows both thighs.
/// Waist and lower leg rings retain their authored/source-section influences.
fn blend_hip_seam(
    legs: [[u16; 2]; 2],
    points: &[Vec3],
    coordinates: &[[f32; 2]],
    faces: &[[usize; 3]],
    joints: &mut [[u16; 4]],
    weights: &mut [[f32; 4]],
) {
    let active: Vec<_> = coordinates
        .iter()
        .map(|uv| {
            let row = (uv[1] - if uv[1] < 12. { 10. } else { 12. }) * ROWS as f32;
            uv[1] >= 10. && row < 4.
        })
        .collect();
    let mut bones: Vec<_> = joints
        .iter()
        .zip(weights.iter())
        .flat_map(|(ids, weights)| {
            (0..4).filter_map(|slot| (weights[slot] > 0.).then_some(ids[slot]))
        })
        .collect();
    bones.sort_unstable();
    bones.dedup();
    let mut field = vec![vec![0.; bones.len()]; points.len()];
    for i in 0..points.len() {
        for slot in 0..4 {
            if weights[i][slot] > 0. {
                field[i][bones.binary_search(&joints[i][slot]).unwrap()] += weights[i][slot];
            }
        }
    }
    let mut edges = BTreeMap::<(usize, usize), f32>::new();
    for &[a, b, c] in faces {
        for (a, b) in [(a, b), (b, c), (c, a)] {
            edges
                .entry((a.min(b), a.max(b)))
                .or_insert_with(|| 1. / points[a].distance(points[b]).max(0.0001));
        }
    }
    let mut neighbors = vec![Vec::new(); points.len()];
    for ((a, b), conductance) in edges {
        neighbors[a].push((b, conductance));
        neighbors[b].push((a, conductance));
    }
    for _ in 0..1024 {
        let mut change = 0_f32;
        for i in (0..points.len()).filter(|&i| active[i]) {
            let total: f32 = neighbors[i].iter().map(|p| p.1).sum();
            for bone in 0..bones.len() {
                let value = neighbors[i]
                    .iter()
                    .map(|&(j, w)| field[j][bone] * w)
                    .sum::<f32>()
                    / total;
                change = change.max((field[i][bone] - value).abs());
                field[i][bone] = value;
            }
        }
        if change < 0.000002 {
            break;
        }
    }
    for i in (0..points.len()).filter(|&i| active[i]) {
        // The saddle can contain hips, both thighs and both shins. Blindly
        // keeping its four largest weights drops a different shin at adjacent
        // vertices and breaks continuity again. Upper panels follow thighs;
        // introduce only their own shin gradually toward the lower leg ring.
        let side = usize::from(coordinates[i][1] >= 12.);
        let row = (coordinates[i][1] - if side == 0 { 10. } else { 12. }) * ROWS as f32;
        for (leg, [thigh, shin]) in legs.into_iter().enumerate() {
            if let (Ok(thigh), Ok(shin)) = (bones.binary_search(&thigh), bones.binary_search(&shin))
            {
                let keep = if side == leg { smooth(row / 4.) } else { 0. };
                let moved = field[i][shin] * (1. - keep);
                field[i][shin] -= moved;
                field[i][thigh] += moved;
            }
        }
        let mut influences: Vec<_> = field[i]
            .iter()
            .copied()
            .enumerate()
            .filter(|&(_, weight)| weight > 0.000001)
            .collect();
        influences.sort_by(|a, b| b.1.total_cmp(&a.1));
        influences.truncate(4);
        let total: f32 = influences.iter().map(|p| p.1).sum();
        joints[i] = std::array::from_fn(|slot| influences.get(slot).map_or(0, |p| bones[p.0]));
        weights[i] = std::array::from_fn(|slot| influences.get(slot).map_or(0., |p| p.1 / total));
    }
}

/// Art trial only until the complete moving-leg contact regression passes.
pub(crate) fn trial_enabled() -> bool {
    #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
    {
        std::env::var_os("CRYSTAL_PEOPLE_REVIEW_TROUSERS_TRIAL").is_some()
            && matches!(
                std::env::var("CRYSTAL_PEOPLE_REVIEW_DRAPE_TRIAL").as_deref(),
                Ok("local" | "live")
            )
    }
    #[cfg(not(all(feature = "people-review", not(target_arch = "wasm32"))))]
    {
        false
    }
}

pub(crate) fn build(rig: &PersonRig, look: Look) -> Option<CostumeMesh> {
    if look.costume != "hakama" {
        return None;
    }
    let mut vertices = BTreeMap::new();
    let mut points = Vec::new();
    let mut joints = Vec::new();
    let mut weights = Vec::new();
    let mut coordinates = Vec::new();
    let mut faces = Vec::new();
    let body = rig.refined_body();
    let cloth_faces: Vec<_> = body
        .mesh
        .indices
        .chunks_exact(3)
        .filter(|face| {
            face.iter().all(|&i| {
                let uv = body.mesh.uvs[i as usize];
                uv[0] >= 0.605 && uv[1] >= 0.745
            })
        })
        .map(|face| [face[0] as usize, face[1] as usize, face[2] as usize])
        .collect();
    assert!(
        !cloth_faces.is_empty(),
        "trouser fit needs the supplied lower body"
    );
    let hips = rig.leg_joints.map(|j| {
        rig.inverse_binds[j[0]]
            .inverse()
            .transform_point3(Vec3::ZERO)
    });
    let knees = rig.leg_joints.map(|j| {
        rig.inverse_binds[j[1]]
            .inverse()
            .transform_point3(Vec3::ZERO)
    });
    // Resolve each ankle from the supplied cuff's actual skin influences,
    // without relying on a particular ordering of the source skin nodes.
    let ankles = std::array::from_fn::<_, 2, _>(|side| {
        let mut candidates = BTreeMap::<usize, f32>::new();
        for (i, (p, uv)) in body.mesh.positions.iter().zip(&body.mesh.uvs).enumerate() {
            if uv[0] < 0.605 || uv[1] < 0.745 || p[1] > 0.38 || (p[0] < 0.) != (side == 1) {
                continue;
            }
            for slot in 0..4 {
                let joint = body.ids[i][slot] as usize;
                if joint != rig.hips_joint && !rig.leg_joints[side].contains(&joint) {
                    *candidates.entry(joint).or_default() += body.weights[i][slot];
                }
            }
        }
        let (joint, weight) = candidates
            .into_iter()
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .expect("trouser cuff must include its ankle skin");
        assert!(weight > 0.);
        rig.inverse_binds[joint]
            .inverse()
            .transform_point3(Vec3::ZERO)
    });
    const SIDES: usize = 32;
    let mut vertex = |p: Vec3, side: usize, coordinate: [f32; 2]| {
        let key = p.to_array().map(|v| (v * 1_000_000.).round() as i32);
        *vertices.entry(key).or_insert_with(|| {
            let index = points.len();
            points.push(p);
            coordinates.push(coordinate);
            // Sample the actual horizontal section, not the nearest folded
            // corner. A wide hip panel can otherwise inherit a lower thigh's
            // weights. The welded crotch averages both source leg sections.
            if coordinate[1] < 10. {
                joints.push([rig.hips_joint as u16, 0, 0, 0]);
                weights.push([1., 0., 0., 0.]);
                return index;
            }
            let sides: &[usize] = if p.x.abs() < 0.00001 {
                &[0, 1]
            } else {
                std::slice::from_ref(&side)
            };
            let mut influences = BTreeMap::<u16, f32>::new();
            for &side in sides {
                for (joint, weight) in section_skin(&body, &cloth_faces, p.y, side) {
                    *influences.entry(joint).or_default() += weight / sides.len() as f32;
                }
            }
            // A wide panel needs a longer bend zone than the close-fitting
            // body's knee weights. Otherwise the rear surface turns inside
            // itself as the same rotation is compressed into a short strip.
            for side in 0..2 {
                let [thigh, shin] = rig.leg_joints[side].map(|j| j as u16);
                let amount = influences.get(&thigh).copied().unwrap_or(0.)
                    + influences.get(&shin).copied().unwrap_or(0.);
                let blend = smooth((p.y - knees[side].y + 0.40) / 0.80);
                influences.insert(thigh, amount * blend);
                influences.insert(shin, amount * (1. - blend));
            }
            let mut influences: Vec<_> = influences.into_iter().filter(|(_, w)| *w > 0.).collect();
            influences.sort_by(|a, b| b.1.total_cmp(&a.1));
            influences.truncate(4);
            let total: f32 = influences.iter().map(|p| p.1).sum();
            joints.push(std::array::from_fn(|slot| {
                influences.get(slot).map_or(0, |p| p.0)
            }));
            weights.push(std::array::from_fn(|slot| {
                influences.get(slot).map_or(0., |p| p.1 / total)
            }));
            index
        })
    };
    let mut top = [[0; SIDES / 2 + 1]; 2];
    for side in 0..2 {
        let sign = if side == 0 { 1. } else { -1. };
        let mut rings = Vec::new();
        for row in 0..=ROWS {
            let mut ring = Vec::new();
            for col in 0..SIDES {
                let angle = std::f32::consts::TAU * col as f32 / SIDES as f32;
                let (sin, cos) = angle.sin_cos();
                // Both panels share the whole curved crotch seam. Their
                // remaining upper half-rings form one continuous waist.
                let mut root = Vec3::new(
                    sign * 0.46 * sin.max(0.),
                    1.36 + 0.36 * sin.min(0.),
                    0.025 + 0.29 * cos,
                );
                if sin >= -0.000001 {
                    let radial = Vec2::new(root.x, root.z - 0.05);
                    if let Some(radius) = section_radius(
                        &body.mesh.positions,
                        &cloth_faces,
                        root.y,
                        Vec2::new(0., 0.05),
                        radial.normalize(),
                        None,
                    ) {
                        let fitted = radial.normalize() * radial.length().max(radius + 0.045);
                        root.x = fitted.x;
                        root.z = fitted.y + 0.05;
                    }
                }
                let t = (row as f32 / 4.).min(1.);
                let y = if row <= 4 {
                    root.y + (0.90 - root.y) * t
                } else {
                    0.90 + (0.26 - 0.90) * (row - 4) as f32 / (ROWS - 4) as f32
                };
                let axis = if y > knees[side].y {
                    hips[side].lerp(
                        knees[side],
                        ((hips[side].y - y) / (hips[side].y - knees[side].y)).clamp(0., 1.),
                    )
                } else {
                    knees[side].lerp(
                        ankles[side],
                        ((knees[side].y - y) / (knees[side].y - ankles[side].y)).clamp(0., 1.),
                    )
                };
                let fullness = smooth((y - 0.26) / 0.40);
                let width = 0.19 + 0.045 * fullness;
                let depth = 0.17 + 0.075 * fullness;
                let pleat = 0.008 * (4. * angle).sin().powi(2) * smooth(t);
                let radial = Vec2::new(sign * sin * (width + pleat), cos * (depth + pleat));
                // The medial seam follows the retained leg envelope; the
                // lateral panel retains the wider authored silhouette.
                let direction = radial.normalize();
                let fitted = section_radius(
                    &body.mesh.positions,
                    &cloth_faces,
                    y,
                    Vec2::new(axis.x, axis.z),
                    direction,
                    Some(side),
                );
                let medial = smooth(-sin);
                let radius = fitted.map_or(radial.length(), |r| {
                    radial.length().lerp((r + 0.026).max(0.12), medial)
                });
                let target = Vec3::new(
                    axis.x + direction.x * radius,
                    y,
                    axis.z + direction.y * radius,
                );
                let p = Vec3::new(
                    // Open the fork immediately. A zero-tangent width at the
                    // saddle leaves nearly coincident panels that cross when
                    // the thighs turn inward, even with positive normals.
                    root.x + (target.x - root.x) * t,
                    y,
                    root.z + (target.z - root.z) * smooth(t),
                );
                let index = vertex(
                    p,
                    side,
                    [
                        col as f32 / SIDES as f32,
                        10. + 2. * side as f32 + row as f32 / ROWS as f32,
                    ],
                );
                if row == 0 && col <= SIDES / 2 {
                    top[side][col] = index;
                }
                ring.push(index);
            }
            if let Some(previous) = rings.last() {
                let previous: &Vec<usize> = previous;
                for col in 0..SIDES {
                    let next = (col + 1) % SIDES;
                    let mut triangles = [
                        [previous[col], ring[col], ring[next]],
                        [previous[col], ring[next], previous[next]],
                    ];
                    if side == 1 {
                        for face in &mut triangles {
                            face.swap(1, 2);
                        }
                    }
                    faces.extend(triangles);
                }
            }
            rings.push(ring);
        }
    }
    // Follow the actual connected outer boundary, including its rear point.
    let mut previous: Vec<_> = top[0]
        .iter()
        .copied()
        .chain(top[1][1..SIDES / 2].iter().rev().copied())
        .collect();
    for row in 1..=4 {
        let t = row as f32 / 4.;
        let mut ring = Vec::new();
        for col in 0..SIDES {
            let angle = std::f32::consts::TAU * col as f32 / SIDES as f32;
            let (sin, cos) = angle.sin_cos();
            let direction = Vec2::new(sin, cos);
            let seam = crate::garment_fit::section(rig, 1.69, direction)
                .expect("trouser waist must meet the supplied torso");
            let root = Vec2::new(sin * 0.46, 0.025 + cos * 0.29);
            let fitted = direction * (seam.radius + 0.045) + Vec2::new(0., 0.05);
            let mut radial = root.lerp(fitted, smooth(t));
            let centered = radial - Vec2::new(0., 0.05);
            if let Some(radius) = section_radius(
                &body.mesh.positions,
                &cloth_faces,
                1.36 + 0.33 * t,
                Vec2::new(0., 0.05),
                centered.normalize(),
                None,
            ) {
                radial = centered.normalize() * centered.length().max(radius + 0.045)
                    + Vec2::new(0., 0.05);
            }
            let p = Vec3::new(radial.x, 1.36 + 0.33 * t, radial.y);
            ring.push(vertex(
                p,
                usize::from(p.x < 0.),
                [col as f32 / SIDES as f32, 8.],
            ));
        }
        for col in 0..SIDES {
            let next = (col + 1) % SIDES;
            faces.extend([
                [previous[col], previous[next], ring[next]],
                [previous[col], ring[next], ring[col]],
            ]);
        }
        previous = ring;
    }
    blend_hip_seam(
        rig.leg_joints.map(|leg| leg.map(|j| j as u16)),
        &points,
        &coordinates,
        &faces,
        &mut joints,
        &mut weights,
    );
    let mut original_normals = vec![Vec3::ZERO; points.len()];
    let mut edges = BTreeMap::<(usize, usize), Vec<(usize, usize)>>::new();
    for face in &faces {
        let p = face.map(|i| points[i]);
        let n = (p[1] - p[0]).cross(p[2] - p[0]);
        for &i in face {
            original_normals[i] += n;
        }
        for (a, b) in [(face[0], face[1]), (face[1], face[2]), (face[2], face[0])] {
            edges.entry((a.min(b), a.max(b))).or_default().push((a, b));
        }
    }
    assert!(
        edges.values().all(|v| v.len() <= 2),
        "non-manifold trouser cut"
    );
    for n in &mut original_normals {
        *n = n.normalize();
    }
    let painting = crate::npc_textures::get(look.texture_key);
    let layer = points.len();
    let mut out = CostumeMesh {
        surface: SurfaceMeshData::default(),
        joints: Vec::with_capacity(layer * 2),
        weights: Vec::with_capacity(layer * 2),
    };
    for inner in [false, true] {
        for (i, &p) in points.iter().enumerate() {
            // The authored panel is the outside of the cloth. Growing both
            // layers outward consumed the narrow crotch space and made the
            // two legs cross before animation. Put actual thickness inside.
            let tailored = p + original_normals[i] * if inner { -0.006 } else { 0.0 };
            out.surface.positions.push(tailored.to_array());
            out.surface.normals.push([0.; 3]);
            let rgb = painting
                .map(|painting| {
                    let (w, h) = painting.size;
                    let x = (0.75 * w as f32) as u32;
                    let y = (0.82 * h as f32) as u32;
                    let at = ((y * w + x) * 4) as usize;
                    [
                        painting.rgba[at],
                        painting.rgba[at + 1],
                        painting.rgba[at + 2],
                    ]
                })
                .unwrap_or(look.lower);
            let mut color = Color::srgb_u8(rgb[0], rgb[1], rgb[2])
                .to_linear()
                .to_f32_array();
            if inner {
                for channel in &mut color[..3] {
                    *channel *= 0.82;
                }
            }
            out.surface.colors.push(color);
            out.surface.uvs.push(coordinates[i]);
            out.joints.push(joints[i]);
            out.weights.push(weights[i]);
        }
    }
    for [a, b, c] in faces {
        out.surface
            .indices
            .extend([a, b, c, c + layer, b + layer, a + layer].map(|i| i as u32));
    }
    for face in out.surface.indices.chunks_exact(3) {
        let p = [face[0], face[1], face[2]]
            .map(|i| Vec3::from_array(out.surface.positions[i as usize]));
        let normal = (p[1] - p[0]).cross(p[2] - p[0]);
        for &i in face {
            let n = &mut out.surface.normals[i as usize];
            *n = (Vec3::from_array(*n) + normal).to_array();
        }
    }
    for normal in &mut out.surface.normals {
        *normal = Vec3::from_array(*normal).normalize().to_array();
    }
    // The waist and cuff cut edges have their own normal islands. Averaging
    // a thin closing rim into the leg surface creates dark flipped highlights.
    for boundary in edges.values().filter(|v| v.len() == 1) {
        let (a, b) = boundary[0];
        for triangle in [[b, a, a + layer], [b, a + layer, b + layer]] {
            let p = triangle.map(|i| Vec3::from_array(out.surface.positions[i]));
            let normal = (p[1] - p[0]).cross(p[2] - p[0]).normalize().to_array();
            for i in triangle {
                out.surface.indices.push(out.surface.positions.len() as u32);
                out.surface.positions.push(out.surface.positions[i]);
                out.surface.normals.push(normal);
                out.surface.colors.push(out.surface.colors[i]);
                out.surface.uvs.push(out.surface.uvs[i]);
                out.joints.push(out.joints[i]);
                out.weights.push(out.weights[i]);
            }
        }
    }
    // Preserve the cut's hard pleat/crotch corners. A smooth normal spanning
    // a deep crease can point behind one of its actual triangles.
    let indices = std::mem::take(&mut out.surface.indices);
    for face in indices.chunks_exact(3) {
        let p = [face[0], face[1], face[2]]
            .map(|i| Vec3::from_array(out.surface.positions[i as usize]));
        let normal = (p[1] - p[0]).cross(p[2] - p[0]).normalize();
        for &i in face {
            let i = i as usize;
            if Vec3::from_array(out.surface.normals[i]).dot(normal) >= 0.5 {
                out.surface.indices.push(i as u32);
            } else {
                out.surface.indices.push(out.surface.positions.len() as u32);
                out.surface.positions.push(out.surface.positions[i]);
                out.surface.normals.push(normal.to_array());
                out.surface.colors.push(out.surface.colors[i]);
                out.surface.uvs.push(out.surface.uvs[i]);
                out.joints.push(out.joints[i]);
                out.weights.push(out.weights[i]);
            }
        }
    }
    Some(out)
}

#[cfg(any(test, all(feature = "people-review", not(target_arch = "wasm32"))))]
pub(crate) fn validate(rig: &PersonRig, look: Look) {
    let trousers = build(rig, look).expect("tailored trousers");
    let mesh = &trousers.surface;
    #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
    crate::garment_drape_trial::validate_closed_fabric(mesh);
    let mut welds = BTreeMap::new();
    let welded: Vec<_> = mesh
        .positions
        .iter()
        .map(|p| {
            let key = p.map(|v| (v * 100_000.).round() as i32);
            let count = welds.len();
            *welds.entry(key).or_insert(count)
        })
        .collect();
    let mut edges = BTreeMap::<(usize, usize), (usize, i32)>::new();
    for face in mesh.indices.chunks_exact(3) {
        let p = [face[0], face[1], face[2]].map(|i| Vec3::from_array(mesh.positions[i as usize]));
        assert!((p[1] - p[0]).cross(p[2] - p[0]).length_squared() > 1e-16);
        let normal = face
            .iter()
            .map(|&i| Vec3::from_array(mesh.normals[i as usize]))
            .sum::<Vec3>();
        assert!(
            (p[1] - p[0]).cross(p[2] - p[0]).dot(normal) > 0.,
            "inward trouser shading"
        );
        for (a, b) in [(face[0], face[1]), (face[1], face[2]), (face[2], face[0])] {
            let (a, b) = (welded[a as usize], welded[b as usize]);
            let edge = edges.entry((a.min(b), a.max(b))).or_default();
            edge.0 += 1;
            edge.1 += if a < b { 1 } else { -1 };
        }
    }
    assert!(
        edges.values().all(|&edge| edge == (2, 0)),
        "open trouser seam"
    );
    // Doubling a surface with one waist and two leg openings has genus two.
    // A single doubled skirt tube has Euler characteristic zero and fails.
    assert_eq!(
        welds.len() as isize - edges.len() as isize + (mesh.indices.len() / 3) as isize,
        -2,
        "requires two separate trouser legs"
    );
    for i in 0..mesh.positions.len() {
        let p = Vec3::from_array(mesh.positions[i]);
        let n = Vec3::from_array(mesh.normals[i]);
        assert!(p.is_finite() && n.is_finite() && (n.length() - 1.).abs() < 1e-4);
        assert!(
            trousers.weights[i]
                .iter()
                .all(|&w| w.is_finite() && w >= 0.)
        );
        assert!((trousers.weights[i].iter().sum::<f32>() - 1.).abs() < 1e-5);
        assert!(
            trousers.joints[i]
                .iter()
                .all(|&j| (j as usize) < rig.skin_nodes.len())
        );
    }
    for sign in [-1., 1.] {
        let width = |low: f32, high: f32| {
            let points: Vec<_> = mesh
                .positions
                .iter()
                .filter(|p| sign * p[0] > 0. && (low..high).contains(&p[1]))
                .collect();
            let low = points.iter().map(|p| p[0]).fold(f32::INFINITY, f32::min);
            let high = points
                .iter()
                .map(|p| p[0])
                .fold(f32::NEG_INFINITY, f32::max);
            high - low
        };
        let cuff = width(0.20, 0.34);
        let calf = width(0.55, 0.75);
        assert!(
            cuff > 0.30 && cuff / calf > 0.70,
            "trouser cuff pinches below a ballooned calf on side {sign}: {cuff}/{calf}"
        );
        assert!(
            mesh.positions
                .iter()
                .any(|p| sign * p[0] > 0.44 && (0.60..1.10).contains(&p[1])),
            "wide trouser silhouette missing on side {sign}"
        );
        assert!(
            mesh.positions
                .iter()
                .any(|p| sign * p[0] > 0.16 && p[1] < 0.43),
            "trouser cuff missing on side {sign}"
        );
    }
    let mut waist_samples = 0;
    for (p, uv) in mesh.positions.iter().zip(&mesh.uvs) {
        if uv[1] != 8. || p[1] < 1.65 {
            continue;
        }
        let radial = Vec2::new(p[0], p[2] - 0.05);
        let sample = crate::garment_fit::section(rig, 1.69, radial.normalize()).unwrap();
        let gap = radial.length() - sample.radius;
        assert!((0.0..0.065).contains(&gap), "detached trouser waist: {gap}");
        waist_samples += 1;
    }
    assert!(waist_samples >= 32, "trousers must reach the authored obi");
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "requires supplied Falkner rig and painting in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_trousers_have_two_articulated_closed_legs() {
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let rig = crate::open_people::PersonRig::parse(
            &std::fs::read(root.join("people/kenney/human-male.glb")).unwrap(),
        )
        .unwrap();
        let mut look = crate::npc_appearance::look(
            crate::new_bark_actors::character_meshes::CharacterKind::Falkner,
        );
        look.costume = "hakama";
        super::validate(&rig, look);
        // The former left/right section transfer flipped two triangles at the
        // connected crotch even in the ordinary production idle. Evaluate the
        // actual animated surface, not only its closed neutral topology.
        let kind = crate::new_bark_actors::character_meshes::CharacterKind::Falkner;
        let locomotion = crate::human_locomotion::HumanLocomotion::for_character(&rig, kind);
        let idle = crate::human_idle::HumanIdle::new(&rig, kind);
        let mut pose = rig.nodes.clone();
        idle.sample(&rig, &locomotion, 0., &mut pose);
        let world = |node| crate::human_locomotion::world(&rig, &pose, node);
        let inverse = world(rig.mesh_node).inverse();
        let palette: Vec<_> = rig
            .skin_nodes
            .iter()
            .zip(&rig.inverse_binds)
            .map(|(&node, bind)| inverse * world(node) * *bind)
            .collect();
        let trousers = super::build(&rig, look).unwrap();
        let matrices: Vec<bevy::prelude::Mat4> = trousers
            .joints
            .iter()
            .zip(&trousers.weights)
            .map(|(joints, weights)| {
                (0..4)
                    .map(|slot| palette[joints[slot] as usize] * weights[slot])
                    .sum()
            })
            .collect();
        let points: Vec<_> = trousers
            .surface
            .positions
            .iter()
            .zip(&matrices)
            .map(|(&p, matrix)| matrix.transform_point3(bevy::prelude::Vec3::from_array(p)))
            .collect();
        let normals: Vec<_> = trousers
            .surface
            .normals
            .iter()
            .zip(&matrices)
            .map(|(&n, matrix)| {
                (bevy::prelude::Mat3::from_mat4(*matrix)
                    .inverse()
                    .transpose()
                    * bevy::prelude::Vec3::from_array(n))
                .normalize()
            })
            .collect();
        #[cfg(all(feature = "people-review", not(target_arch = "wasm32")))]
        crate::garment_drape_trial::validate_closed_fabric(&crate::mesh::SurfaceMeshData {
            positions: points.iter().map(|p| p.to_array()).collect(),
            indices: trousers.surface.indices.clone(),
            ..Default::default()
        });
        for face in trousers.surface.indices.chunks_exact(3) {
            let [a, b, c] = [face[0], face[1], face[2]].map(|i| i as usize);
            let geometric = (points[b] - points[a]).cross(points[c] - points[a]);
            assert!(
                geometric.dot(normals[a] + normals[b] + normals[c]) > 0.,
                "production idle folded trouser face {face:?}"
            );
        }
    }
}
