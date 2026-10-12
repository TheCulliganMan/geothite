//! Small uniform panels sampled from source torso triangles and skin weights.
use crate::{
    mesh::SurfaceMeshData,
    npc_appearance::{CostumeMesh, Look},
    open_people::PersonRig,
};
use bevy::prelude::*;

fn sample(rig: &PersonRig, at: Vec2, side: f32) -> Option<(Vec3, Vec3, [u16; 4], [f32; 4])> {
    let mut result = None;
    let mut depth = f32::NEG_INFINITY;
    for face in rig.mesh.indices.chunks_exact(3) {
        let ids = [face[0] as usize, face[1] as usize, face[2] as usize];
        if ids.iter().all(|&i| rig.mesh.positions[i][1] < 1.30)
            || ids.iter().all(|&i| rig.mesh.positions[i][1] > 2.56)
        {
            continue;
        }
        let [a, b, c] = ids.map(|i| Vec3::from_array(rig.mesh.positions[i]));
        let denominator = (b.y - c.y) * (a.x - c.x) + (c.x - b.x) * (a.y - c.y);
        if denominator.abs() < 1e-8 {
            continue;
        }
        let u = ((b.y - c.y) * (at.x - c.x) + (c.x - b.x) * (at.y - c.y)) / denominator;
        let v = ((c.y - a.y) * (at.x - c.x) + (a.x - c.x) * (at.y - c.y)) / denominator;
        if u < -1e-5 || v < -1e-5 || u + v > 1.00001 {
            continue;
        }
        let bary = [u.max(0.), v.max(0.), (1. - u - v).max(0.)];
        let p = a * bary[0] + b * bary[1] + c * bary[2];
        if p.z * side <= depth {
            continue;
        }
        depth = p.z * side;
        let normal: Vec3 = (0..3)
            .map(|i| Vec3::from_array(rig.mesh.normals[ids[i]]) * bary[i])
            .sum();
        let mut blend = std::collections::BTreeMap::<u16, f32>::new();
        for i in 0..3 {
            for slot in 0..4 {
                *blend.entry(rig.ids[ids[i]][slot]).or_default() +=
                    rig.weights[ids[i]][slot] * bary[i];
            }
        }
        let mut blend: Vec<_> = blend.into_iter().collect();
        blend.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        blend.truncate(4);
        let total: f32 = blend.iter().map(|p| p.1).sum();
        result = Some((
            p + Vec3::Z * (0.026 * side),
            normal.normalize(),
            std::array::from_fn(|i| blend.get(i).map_or(0, |p| p.0)),
            std::array::from_fn(|i| blend.get(i).map_or(0., |p| p.1 / total)),
        ));
    }
    result
}
fn triangle(out: &mut CostumeMesh, rig: &PersonRig, p: [Vec2; 3], side: f32, rgb: [u8; 3]) {
    let edges = [(0, 1, 2), (1, 2, 0), (2, 0, 1)];
    let &(a, b, c) = edges
        .iter()
        .max_by(|&&(a, b, _), &&(c, d, _)| {
            p[a].distance_squared(p[b])
                .total_cmp(&p[c].distance_squared(p[d]))
        })
        .unwrap();
    if p[a].distance_squared(p[b]) > 0.045 * 0.045 {
        let mid = (p[a] + p[b]) * 0.5;
        triangle(out, rig, [p[a], mid, p[c]], side, rgb);
        triangle(out, rig, [mid, p[b], p[c]], side, rgb);
        return;
    }
    let Some(vertices) = p
        .map(|p| sample(rig, p, side))
        .into_iter()
        .collect::<Option<Vec<_>>>()
    else {
        return;
    };
    let geometric = (vertices[1].0 - vertices[0].0).cross(vertices[2].0 - vertices[0].0);
    if geometric.length_squared() < 1e-12 {
        return;
    }
    let order = if geometric.z * side > 0. {
        [0, 1, 2]
    } else {
        [0, 2, 1]
    };
    let rgb = rgb.map(|v| Color::srgb_u8(v, v, v).to_linear().red);
    let first = out.surface.positions.len() as u32;
    for i in order {
        let (p, n, j, w) = vertices[i];
        out.surface.positions.push(p.to_array());
        out.surface.normals.push(n.to_array());
        out.surface.uvs.push([0.; 2]);
        out.surface.colors.push([rgb[0], rgb[1], rgb[2], 1.]);
        out.joints.push(j);
        out.weights.push(w);
    }
    out.surface.indices.extend([first, first + 1, first + 2]);
}
fn panel(out: &mut CostumeMesh, rig: &PersonRig, points: &[[f32; 2]], side: f32, rgb: [u8; 3]) {
    let center: Vec2 =
        points.iter().map(|&p| Vec2::from_array(p)).sum::<Vec2>() / points.len() as f32;
    for i in 0..points.len() {
        triangle(
            out,
            rig,
            [
                center,
                Vec2::from_array(points[i]),
                Vec2::from_array(points[(i + 1) % points.len()]),
            ],
            side,
            rgb,
        );
    }
}
fn rectangle(out: &mut CostumeMesh, rig: &PersonRig, b: [f32; 4], side: f32, rgb: [u8; 3]) {
    panel(
        out,
        rig,
        &[[b[0], b[2]], [b[1], b[2]], [b[1], b[3]], [b[0], b[3]]],
        side,
        rgb,
    );
}
fn belt(out: &mut CostumeMesh, rig: &PersonRig) -> Option<()> {
    waist_band(out, rig, 1.45, 0.185, [30, 31, 36])
}
fn waist_band(out: &mut CostumeMesh, rig: &PersonRig, bottom: f32, height: f32, color: [u8; 3]) -> Option<()> {
    const SIDES: usize = 64;
    const ROWS: usize = 6;
    let first = out.surface.positions.len() as u32;
    let rgb = Color::srgb_u8(color[0], color[1], color[2]).to_linear();
    for row in 0..=ROWS {
        for col in 0..=SIDES {
            let y = bottom + height * row as f32 / ROWS as f32;
            let a = std::f32::consts::TAU * col as f32 / SIDES as f32;
            let direction = Vec2::new(a.sin(), a.cos());
            let sample = crate::garment_fit::section(rig, y, direction)?;
            let radius = sample.radius + 0.018;
            out.surface
                .positions
                .push([direction.x * radius, y, 0.05 + direction.y * radius]);
            out.surface.normals.push([direction.x, 0., direction.y]);
            out.surface.uvs.push([0.; 2]);
            out.surface.colors.push([rgb.red, rgb.green, rgb.blue, 1.]);
            let mut influences = sample.influences;
            influences.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
            influences.truncate(4);
            let total: f32 = influences.iter().map(|p| p.1).sum();
            out.joints.push(std::array::from_fn(|i| {
                influences.get(i).map_or(0, |p| p.0 as u16)
            }));
            out.weights.push(std::array::from_fn(|i| {
                influences.get(i).map_or(0., |p| p.1 / total)
            }));
        }
    }
    for row in 0..ROWS {
        for col in 0..SIDES {
            let a = first + (row * (SIDES + 1) + col) as u32;
            let b = a + (SIDES + 1) as u32;
            out.surface.indices.extend([a, a + 1, b, a + 1, b + 1, b]);
        }
    }
    Some(())
}

/// An open sleeveless vest, sampled around the torso rather than painted onto
/// the shared shirt UVs. Lower side edges join the back continuously; the upper
/// edge drops beneath each armpit so it cannot bridge onto the moving sleeves.
fn biker_vest(out: &mut CostumeMesh, rig: &PersonRig) -> Option<()> {
    const SIDES: usize = 64;
    const ROWS: usize = 24;
    let first = out.surface.positions.len() as u32;
    let opening = 0.24;
    let rgb = Color::srgb_u8(42, 43, 47).to_linear();
    for row in 0..=ROWS {
        for col in 0..=SIDES {
            let angle = opening + (std::f32::consts::TAU - 2. * opening)
                * col as f32 / SIDES as f32;
            let direction = Vec2::new(angle.sin(), angle.cos());
            let top = 2.40 - 0.37 * direction.x.powi(2);
            let y = 1.50 + (top - 1.50) * row as f32 / ROWS as f32;
            let sample = crate::garment_fit::section(rig, y, direction)?;
            let radius = sample.radius + 0.018;
            out.surface.positions.push([direction.x * radius, y, 0.05 + direction.y * radius]);
            out.surface.normals.push([direction.x, 0., direction.y]);
            out.surface.uvs.push([0.; 2]);
            out.surface.colors.push([rgb.red, rgb.green, rgb.blue, 1.]);
            let mut influences = sample.influences;
            influences.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
            influences.truncate(4);
            let total: f32 = influences.iter().map(|p| p.1).sum();
            out.joints.push(std::array::from_fn(|i| influences.get(i).map_or(0, |p| p.0 as u16)));
            out.weights.push(std::array::from_fn(|i| influences.get(i).map_or(0., |p| p.1 / total)));
        }
    }
    for row in 0..ROWS {
        for col in 0..SIDES {
            let a = first + (row * (SIDES + 1) + col) as u32;
            let b = a + (SIDES + 1) as u32;
            out.surface.indices.extend([a, a + 1, b, a + 1, b + 1, b]);
        }
    }
    // The source torso is not cylindrical. Shade the fitted surface itself,
    // including its shoulder slope, instead of assigning radial belt normals.
    let mut normals = vec![Vec3::ZERO; out.surface.positions.len() - first as usize];
    for face in out.surface.indices.chunks_exact(3).filter(|f| f[0] >= first) {
        let p = [face[0], face[1], face[2]]
            .map(|i| Vec3::from_array(out.surface.positions[i as usize]));
        let normal = (p[1] - p[0]).cross(p[2] - p[0]);
        for &i in face { normals[i as usize - first as usize] += normal; }
    }
    for (i, normal) in normals.into_iter().enumerate() {
        if let Some(normal) = normal.try_normalize() {
            out.surface.normals[first as usize + i] = normal.to_array();
        }
    }
    Some(())
}

fn cardigan_details(rig: &PersonRig, look: Look) -> Option<CostumeMesh> {
    if look.texture_key != "teacher" { return None; }
    let mut out = CostumeMesh {
        surface: SurfaceMeshData::default(), joints: Vec::new(), weights: Vec::new(),
    };
    // The cardigan is layered over a blouse. These front-only details must
    // not repeat on the shared sleeve/back UVs or replace her separate skirt.
    panel(&mut out, rig, &[
        [-0.15, 2.43], [0.15, 2.43], [0., 2.10],
    ], 1., [234, 233, 220]);
    let trim = look.shirt.map(|channel| (channel as f32 * 0.78).round() as u8);
    for sign in [-1., 1.] {
        let first = out.surface.positions.len();
        panel(&mut out, rig, &[
            [0.15 * sign, 2.43], [0.18 * sign, 2.41],
            [0.025 * sign, 2.10], [0., 2.10],
        ], 1., trim);
        for p in &mut out.surface.positions[first..] { p[2] += 0.006; }
    }
    rectangle(&mut out, rig, [-0.018, 0.018, 1.62, 2.10], 1., trim);
    for y in [1.76, 1.94, 2.10] {
        let points: Vec<_> = (0..12).map(|i| {
            let a = std::f32::consts::TAU * i as f32 / 12.;
            [0.018 * a.cos(), y + 0.018 * a.sin()]
        }).collect();
        let first = out.surface.positions.len();
        panel(&mut out, rig, &points, 1., [222, 204, 166]);
        for p in &mut out.surface.positions[first..] { p[2] += 0.012; }
    }
    Some(out)
}

fn rocket_details(rig: &PersonRig, look: Look) -> Option<CostumeMesh> {
    if !matches!(look.texture_key, "rocket" | "rocket_girl") { return None; }
    let mut out = CostumeMesh {
        surface: SurfaceMeshData::default(), joints: Vec::new(), weights: Vec::new(),
    };
    // A fitted chest emblem leaves the shared back/sleeve atlas unmarked.
    // Separate convex strokes keep the counter open without overlapping faces.
    let red = [209, 43, 57];
    rectangle(&mut out, rig, [-0.16, -0.08, 1.90, 2.35], 1., red);
    panel(&mut out, rig, &[
        [-0.08, 2.35], [0.07, 2.35], [0.14, 2.29], [0.07, 2.27], [-0.08, 2.27],
    ], 1., red);
    panel(&mut out, rig, &[
        [0.07, 2.27], [0.14, 2.29], [0.14, 2.12], [0.07, 2.14],
    ], 1., red);
    panel(&mut out, rig, &[
        [-0.08, 2.06], [0.07, 2.06], [0.14, 2.12], [0.07, 2.14], [-0.08, 2.14],
    ], 1., red);
    panel(&mut out, rig, &[
        [0.01, 2.06], [0.08, 2.06], [0.19, 1.90], [0.10, 1.90],
    ], 1., red);
    Some(out)
}

pub(crate) fn append(cloth: &mut CostumeMesh, rig: &PersonRig, look: Look) {
    let extra = cardigan_details(rig, look).or_else(|| {
        // The male uniform is returned by build; the female one composes
        // here so its skirt stays in the same skinned costume.
        if look.texture_key == "rocket_girl" { rocket_details(rig, look) } else { None }
    });
    let Some(extra) = extra else { return; };
    let first = cloth.surface.positions.len() as u32;
    cloth.surface.positions.extend(extra.surface.positions);
    cloth.surface.normals.extend(extra.surface.normals);
    cloth.surface.colors.extend(extra.surface.colors);
    cloth.surface.uvs.extend(extra.surface.uvs);
    cloth.surface.indices.extend(extra.surface.indices.into_iter().map(|i| i + first));
    cloth.joints.extend(extra.joints);
    cloth.weights.extend(extra.weights);
}

pub(crate) fn build(rig: &PersonRig, look: Look) -> Option<CostumeMesh> {
    if look.texture_key == "rocket" { return rocket_details(rig, look); }
    if !matches!(look.texture_key, "officer" | "captain" | "sailor" | "clerk" | "biker" | "black_belt" | "gentleman" | "gym_guide" | "unused_guy" | "cal") {
        return None;
    }
    let mut out = CostumeMesh {
        surface: SurfaceMeshData::default(),
        joints: Vec::new(),
        weights: Vec::new(),
    };
    if look.texture_key == "cal" {
        // A zip jacket needs a continuous hem around the sides and back,
        // rather than front-only stripes on the shared sleeve/body UV island.
        waist_band(&mut out, rig, 1.46, 0.085, [137, 35, 43])?;
        rectangle(&mut out, rig, [-0.042, 0.042, 1.545, 2.36], 1., [81, 30, 39]);
        let first = out.surface.positions.len();
        rectangle(&mut out, rig, [-0.009, 0.009, 1.565, 2.33], 1., [179, 169, 148]);
        for p in &mut out.surface.positions[first..] { p[2] += 0.006; }
        for sign in [-1., 1.] {
            let first = out.surface.positions.len();
            let collar = [[0.015, 2.34], [0.12, 2.43], [0.25, 2.34], [0.12, 2.23]]
                .map(|[x, y]| [x * sign, y]);
            panel(&mut out, rig, &collar, 1., [164, 39, 47]);
            for p in &mut out.surface.positions[first..] { p[2] += 0.012; }
            let pocket = [[0.11, 1.73], [0.24, 1.84], [0.26, 1.815], [0.13, 1.705]]
                .map(|[x, y]| [x * sign, y]);
            panel(&mut out, rig, &pocket, 1., [116, 34, 44]);
        }
        let first = out.surface.positions.len();
        rectangle(&mut out, rig, [-0.02, 0.02, 2.24, 2.30], 1., [193, 181, 151]);
        for p in &mut out.surface.positions[first..] { p[2] += 0.014; }
        return Some(out);
    }
    if matches!(look.texture_key, "gentleman" | "gym_guide" | "unused_guy") {
        let ochre_suit = look.texture_key == "unused_guy";
        let opening = if ochre_suit { [105, 76, 40] } else { [28, 44, 67] };
        if ochre_suit {
            // The jacket hem closes around the torso instead of ending as
            // disconnected front details on the shared shirt painting.
            waist_band(&mut out, rig, 1.46, 0.055, [150, 113, 57])?;
        }
        // Keep front-specific suit details off the shared shirt UV island.
        // Every panel follows the same torso surface and skin influences.
        panel(&mut out, rig, &[
            [-0.13, 2.43], [0.13, 2.43], [0., 1.96],
        ], 1., [234, 233, 220]);
        for sign in [-1., 1.] {
            let first = out.surface.positions.len();
            let points = [[0.13, 2.43], [0.29, 2.30], [0.22, 2.21], [0., 1.96]]
                .map(|[x, y]| [x * sign, y]);
            let lapel = look.shirt.map(|channel| (channel as f32 * 0.82).round() as u8);
            panel(&mut out, rig, &points, 1., lapel);
            for p in &mut out.surface.positions[first..] { p[2] += 0.006; }
        }
        if look.texture_key == "gentleman" {
            // The two bow wings and knot are raised above the ivory shirt.
            for sign in [-1., 1.] {
                let first = out.surface.positions.len();
                let points = [[0.018, 2.32], [0.13, 2.38], [0.13, 2.26]]
                    .map(|[x, y]| [x * sign, y]);
                panel(&mut out, rig, &points, 1., [28, 36, 49]);
                for p in &mut out.surface.positions[first..] { p[2] += 0.012; }
            }
            let first = out.surface.positions.len();
            rectangle(&mut out, rig, [-0.025, 0.025, 2.29, 2.35], 1., [43, 51, 62]);
            for p in &mut out.surface.positions[first..] { p[2] += 0.016; }
        } else {
            let first = out.surface.positions.len();
            panel(&mut out, rig, &[
                [0., 2.34], [-0.032, 2.29], [-0.024, 2.06],
                [0., 2.00], [0.024, 2.06], [0.032, 2.29],
            ], 1., if ochre_suit { [45, 117, 119] } else { [145, 49, 63] });
            for p in &mut out.surface.positions[first..] { p[2] += 0.012; }
        }
        rectangle(&mut out, rig, [-0.014, 0.014, 1.50, 1.94], 1., opening);
        rectangle(&mut out, rig, [-0.24, -0.10, 2.08, 2.105], 1., opening);
        if ochre_suit {
            for sign in [-1., 1.] {
                let pocket = [[0.10, 1.72], [0.25, 1.72], [0.25, 1.695], [0.10, 1.695]]
                    .map(|[x, y]| [x * sign, y]);
                panel(&mut out, rig, &pocket, 1., opening);
            }
        }
        for y in [1.64, 1.81] {
            let points: Vec<_> = (0..12).map(|i| {
                let a = std::f32::consts::TAU * i as f32 / 12.;
                [0.015 * a.cos(), y + 0.015 * a.sin()]
            }).collect();
            let first = out.surface.positions.len();
            panel(&mut out, rig, &points, 1., [73, 83, 97]);
            for p in &mut out.surface.positions[first..] { p[2] += 0.006; }
        }
        return Some(out);
    }
    if look.texture_key == "biker" {
        biker_vest(&mut out, rig)?;
        return Some(out);
    }
    if look.texture_key == "black_belt" {
        // Wrapped gi lapels are front-only geometry: painting them on the
        // source shirt would repeat the lapels on the sleeves and back.
        for sign in [-1., 1.] {
            let first = out.surface.positions.len();
            // The inner collar ends at the wrap; carrying both strips to
            // the waist produced an X emblem rather than layered clothing.
            let points = if sign < 0. {
                [[-0.09, 2.43], [-0.20, 2.38], [0.015, 2.20], [0.075, 2.20]]
            } else {
                [[0.09, 2.43], [0.20, 2.38], [-0.16, 1.65], [-0.23, 1.65]]
            };
            panel(&mut out, rig, &points, 1., [47, 63, 82]);
            if sign > 0. {
                // Raise the outer lapel at its crossing to avoid z fighting.
                for p in &mut out.surface.positions[first..] { p[2] += 0.007; }
            }
        }
        belt(&mut out, rig)?;
        let first = out.surface.positions.len();
        rectangle(&mut out, rig, [-0.045, 0.045, 1.49, 1.60], 1., [39, 40, 43]);
        for p in &mut out.surface.positions[first..] { p[2] += 0.006; }
        // Two tapered cloth ends distinguish a tied martial belt from a
        // plain waist stripe. Sample the torso skin palette so the ends
        // follow the same bend as the knot instead of hanging rigidly.
        for points in [
            [[-0.040, 1.53], [0.005, 1.53], [-0.035, 1.32], [-0.085, 1.34]],
            [[0.005, 1.53], [0.045, 1.53], [0.100, 1.37], [0.055, 1.35]],
        ] {
            let first = out.surface.positions.len();
            panel(&mut out, rig, &points, 1., [30, 31, 36]);
            for p in &mut out.surface.positions[first..] { p[2] += 0.009; }
        }
        return Some(out);
    }
    if look.texture_key == "clerk" {
        // A fitted waist-length work apron. Every panel samples the actual
        // torso triangles and their skin palette, including the back ties.
        let cloth = [222, 210, 178];
        panel(&mut out, rig, &[
            [-0.15, 2.34], [0.15, 2.34], [0.29, 1.50], [-0.29, 1.50],
        ], 1., cloth);
        for sign in [-1., 1.] {
            // Meet the bib edge without overlapping coplanar cloth faces.
            rectangle(&mut out, rig, [0.10 * sign - 0.025, 0.10 * sign + 0.025, 2.34, 2.47], 1., cloth);
        }
        waist_band(&mut out, rig, 1.50, 0.055, [205, 192, 159])?;
        let pocket_start = out.surface.positions.len();
        rectangle(&mut out, rig, [-0.115, 0.115, 1.68, 1.88], 1., [205, 192, 159]);
        for p in &mut out.surface.positions[pocket_start..] { p[2] += 0.009; }
        let badge_start = out.surface.positions.len();
        rectangle(&mut out, rig, [0.055, 0.125, 2.20, 2.235], 1., [211, 175, 80]);
        for p in &mut out.surface.positions[badge_start..] { p[2] += 0.009; }
        return Some(out);
    }
    let sailor = look.texture_key == "sailor";
    let captain = look.texture_key == "captain";
    let navy = [34, 54, 88];
    let gold = [215, 176, 73];
    for side in [-1., 1.] {
        let collar = if sailor {
            vec![
                [0.10, 2.43],
                [0.26, 2.31],
                [0.19, 2.12],
                [0., 1.98],
                [0., 2.15],
            ]
        } else {
            vec![[0.085, 2.43], [0.265, 2.35], [0.165, 2.13], [0., 2.29]]
        };
        let collar: Vec<_> = collar.into_iter().map(|[x, y]| [x * side, y]).collect();
        panel(
            &mut out,
            rig,
            &collar,
            1.,
            if sailor {
                navy
            } else if captain {
                [234, 233, 220]
            } else {
                [62, 88, 128]
            },
        );
    }
    if sailor {
        let start = out.surface.positions.len();
        panel(
            &mut out,
            rig,
            &[[0., 2.18], [-0.047, 2.09], [0., 1.86], [0.047, 2.09]],
            1.,
            [161, 46, 53],
        );
        for p in &mut out.surface.positions[start..] {
            p[2] += 0.006;
        }
        rectangle(&mut out, rig, [-0.29, 0.29, 2.10, 2.38], -1., navy);
        let start = out.surface.positions.len();
        rectangle(
            &mut out,
            rig,
            [-0.24, 0.24, 2.135, 2.15],
            -1.,
            [228, 228, 211],
        );
        for p in &mut out.surface.positions[start..] {
            p[2] -= 0.006;
        }
    } else {
        rectangle(&mut out, rig, [-0.018, 0.018, 1.72, 2.19], 1., [26, 42, 65]);
        for y in [1.80, 1.97, 2.14] {
            for x in if captain { vec![-0.11, 0.11] } else { vec![0.] } {
                let points: Vec<_> = (0..12)
                    .map(|i| {
                        let a = std::f32::consts::TAU * i as f32 / 12.;
                        [x + 0.016 * a.cos(), y + 0.016 * a.sin()]
                    })
                    .collect();
                let start = out.surface.positions.len();
                panel(&mut out, rig, &points, 1., gold);
                for p in &mut out.surface.positions[start..] {
                    p[2] += 0.006;
                }
            }
        }
        for sign in [-1., 1.] {
            let start = out.surface.positions.len();
            rectangle(
                &mut out,
                rig,
                [0.18 * sign - 0.040, 0.18 * sign + 0.040, 2.285, 2.32],
                1.,
                if captain { gold } else { navy },
            );
            for p in &mut out.surface.positions[start..] {
                p[2] += 0.006;
            }
        }
        if !captain {
            panel(
                &mut out,
                rig,
                &[
                    [0.14, 2.145],
                    [0.20, 2.155],
                    [0.24, 2.125],
                    [0.225, 2.05],
                    [0.19, 2.02],
                    [0.15, 2.05],
                ],
                1.,
                gold,
            );
            rectangle(
                &mut out,
                rig,
                [-0.25, -0.09, 1.985, 2.008],
                1.,
                [64, 83, 111],
            );
        }
        belt(&mut out, rig)?;
        let start = out.surface.positions.len();
        rectangle(&mut out, rig, [-0.045, 0.045, 1.485, 1.600], 1., gold);
        for p in &mut out.surface.positions[start..] {
            p[2] += 0.006;
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn world(i: usize, pose: &[Transform], parents: &[Option<usize>]) -> Mat4 {
        parents[i]
            .map(|p| world(p, pose, parents))
            .unwrap_or(Mat4::IDENTITY)
            * pose[i].to_matrix()
    }
    fn closest(p: Vec3, triangle: [Vec3; 3]) -> Vec3 {
        let [a, b, c] = triangle;
        let ab = b - a;
        let ac = c - a;
        let n = ab.cross(ac);
        let q = p - n * (n.dot(p - a) / n.length_squared());
        let v = q - a;
        let d = ab.length_squared() * ac.length_squared() - ab.dot(ac).powi(2);
        let u = (ac.length_squared() * v.dot(ab) - ab.dot(ac) * v.dot(ac)) / d;
        let w = (ab.length_squared() * v.dot(ac) - ab.dot(ac) * v.dot(ab)) / d;
        if u >= 0. && w >= 0. && u + w <= 1. {
            return q;
        }
        [(a, b), (b, c), (c, a)]
            .map(|(a, b)| {
                a + (b - a) * ((p - a).dot(b - a) / (b - a).length_squared()).clamp(0., 1.)
            })
            .into_iter()
            .min_by(|a, b| a.distance_squared(p).total_cmp(&b.distance_squared(p)))
            .unwrap()
    }
    #[test]
    #[ignore = "requires locally supplied Kenney rigs in CRYSTAL_OPEN_MODEL_ROOT"]
    fn supplied_uniform_details_are_fitted_and_follow_the_animated_torso() {
        use crate::new_bark_actors::character_meshes::CharacterKind;
        let root = std::path::PathBuf::from(std::env::var_os("CRYSTAL_OPEN_MODEL_ROOT").unwrap());
        let male =
            PersonRig::parse(&std::fs::read(root.join("people/kenney/human-male.glb")).unwrap())
                .unwrap();
        let female =
            PersonRig::parse(&std::fs::read(root.join("people/kenney/human-female.glb")).unwrap())
                .unwrap();
        for kind in [
            CharacterKind::Biker,
            CharacterKind::BlackBelt,
            CharacterKind::Clerk,
            CharacterKind::Officer,
            CharacterKind::Captain,
            CharacterKind::Sailor,
            CharacterKind::Gentleman,
            CharacterKind::GymGuide,
            CharacterKind::UnusedGuy,
            CharacterKind::Cal,
            CharacterKind::Teacher,
            CharacterKind::Rocket,
            CharacterKind::RocketGirl,
        ] {
            let look = crate::npc_appearance::look(kind);
            let composed_details = matches!(kind, CharacterKind::Teacher | CharacterKind::RocketGirl);
            let rig = if composed_details { &female } else { &male };
            let cloth = if composed_details {
                let composed = crate::npc_appearance::costume_mesh(rig, look).unwrap();
                assert!(composed.surface.positions.iter().any(|p| p[1] < 1.1),
                    "{} details must retain the separate skirt", look.texture_key);
                cardigan_details(rig, look).or_else(|| rocket_details(rig, look)).unwrap()
            } else { crate::npc_appearance::costume_mesh(rig, look).unwrap() };
            let minimum_vertices = if matches!(kind, CharacterKind::Gentleman | CharacterKind::GymGuide | CharacterKind::UnusedGuy | CharacterKind::Teacher | CharacterKind::Rocket | CharacterKind::RocketGirl) { 100 } else { 1000 };
            assert!(cloth.surface.positions.len() > minimum_vertices);
            for (j, w) in cloth.joints.iter().zip(&cloth.weights) {
                assert!(j.iter().all(|&id| (id as usize) < rig.skin_nodes.len()));
                assert!(w.iter().all(|w| w.is_finite() && *w >= 0.));
                assert!((w.iter().sum::<f32>() - 1.).abs() < 1e-5);
            }
            for face in cloth.surface.indices.chunks_exact(3) {
                let p = [face[0], face[1], face[2]]
                    .map(|i| Vec3::from_array(cloth.surface.positions[i as usize]));
                let n = (p[1] - p[0]).cross(p[2] - p[0]);
                let shading: Vec3 = face
                    .iter()
                    .map(|&i| Vec3::from_array(cloth.surface.normals[i as usize]))
                    .sum();
                assert!(
                    n.is_finite() && n.length_squared() > 1e-12 && n.dot(shading) > 0.,
                    "{} invalid face {:?}: dot={} area={} normals={:?}",
                    look.texture_key,
                    p,
                    n.dot(shading),
                    n.length_squared(),
                    shading
                );
            }
            for step in 0..16 {
                let mut pose = rig.nodes.clone();
                rig.sample("Run", rig.duration("Run") * step as f32 / 16., &mut pose);
                let inverse = world(rig.mesh_node, &pose, &rig.parents).inverse();
                let skin: Vec<_> = rig
                    .skin_nodes
                    .iter()
                    .zip(&rig.inverse_binds)
                    .map(|(&node, bind)| inverse * world(node, &pose, &rig.parents) * *bind)
                    .collect();
                let deform = |p: [f32; 3], j: [u16; 4], w: [f32; 4]| {
                    (0..4)
                        .map(|i| skin[j[i] as usize].transform_point3(Vec3::from_array(p)) * w[i])
                        .sum::<Vec3>()
                };
                let body: Vec<_> = rig
                    .mesh
                    .positions
                    .iter()
                    .zip(&rig.ids)
                    .zip(&rig.weights)
                    .map(|((&p, &j), &w)| deform(p, j, w))
                    .collect();
                let triangles: Vec<_> = rig
                    .mesh
                    .indices
                    .chunks_exact(3)
                    .map(|f| {
                        [
                            body[f[0] as usize],
                            body[f[1] as usize],
                            body[f[2] as usize],
                        ]
                    })
                    .filter(|p| (p[1] - p[0]).cross(p[2] - p[0]).length_squared() > 1e-12)
                    .collect();
                for i in (0..cloth.surface.positions.len()).step_by(17) {
                    let p = deform(
                        cloth.surface.positions[i],
                        cloth.joints[i],
                        cloth.weights[i],
                    );
                    let (q, n) = triangles
                        .iter()
                        .map(|&t| (closest(p, t), (t[1] - t[0]).cross(t[2] - t[0]).normalize()))
                        .min_by(|a, b| a.0.distance_squared(p).total_cmp(&b.0.distance_squared(p)))
                        .unwrap();
                    assert!(
                        p.is_finite() && p.distance(q) < 0.055,
                        "{} floats from body at {step}: {}",
                        look.texture_key,
                        p.distance(q)
                    );
                    let nearest_source = if (p - q).dot(n) <= -0.002 {
                        rig.mesh
                            .indices
                            .chunks_exact(3)
                            .filter(|f| {
                                (body[f[1] as usize] - body[f[0] as usize])
                                    .cross(body[f[2] as usize] - body[f[0] as usize])
                                    .length_squared()
                                    > 1e-12
                            })
                            .min_by(|a, b| {
                                let hit = |f: &[u32]| {
                                    closest(
                                        p,
                                        [
                                            body[f[0] as usize],
                                            body[f[1] as usize],
                                            body[f[2] as usize],
                                        ],
                                    )
                                    .distance_squared(p)
                                };
                                hit(a).total_cmp(&hit(b))
                            })
                            .map(|f| {
                                f.iter()
                                    .map(|&i| rig.mesh.positions[i as usize])
                                    .collect::<Vec<_>>()
                            })
                    } else {
                        None
                    };
                    assert!(
                        (p - q).dot(n) > -0.002,
                        "{} intersects body at {step}: {}, bind={:?}, animated={:?}, closest={:?}, source={:?}",
                        look.texture_key,
                        (p - q).dot(n),
                        cloth.surface.positions[i],
                        p,
                        q,
                        nearest_source
                    );
                }
            }
        }
        assert!(build(&male, crate::npc_appearance::look(CharacterKind::Trainer)).is_none());
    }
}
