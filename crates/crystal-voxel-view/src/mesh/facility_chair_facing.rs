//! Face authenticated adjacent serving surfaces, without moving floor or actors.
use super::*;

// Keep target selection and the actual L-counter geometry on the same courses.
pub(super) const RECEPTION_COURSES: &[[f32; 4]] = &[[0., 80., 48., 64.], [80., 96., 8., 64.]];

pub(super) fn choose(chair: &source::Match, all: &[source::Match]) -> (u8, Option<source::Match>) {
    if chair.kind() != Kind::Chair {
        return (0, None);
    }
    let center = [chair.column as f32 * 8. + 8., chair.row as f32 * 8. + 4.];
    let mut best: Option<(f32, u8, source::Match)> = None;
    for target in all {
        let boxes = match target.kind() {
            Kind::SquareTable | Kind::MeetingTable | Kind::SideDesk => {
                vec![target.kind().fitting().0]
            }
            Kind::ReceptionCounter => RECEPTION_COURSES.to_vec(),
            _ => continue,
        };
        for [w, e, n, s] in boxes {
            let x = target.column as f32 * 8.;
            let z = target.row as f32 * 8.;
            let dx = center[0].clamp(x + w, x + e) - center[0];
            let dz = center[1].clamp(z + n, z + s) - center[1];
            let distance = dx * dx + dz * dz;
            // These paired seats are at most two gameplay tiles from their
            // serving edge. Farther waiting seats must not aim through a wall
            // toward an unrelated table in the next room.
            if distance <= 0.0001 || distance > 32. * 32. {
                continue;
            }
            let turn = if dx.abs() > dz.abs() {
                if dx > 0. {
                    1
                } else {
                    3
                }
            } else if dz < 0. {
                2
            } else {
                0
            };
            if best.as_ref().is_none_or(|(old, _, _)| distance < *old) {
                best = Some((distance, turn, target.clone()));
            }
        }
    }
    best.map_or((0, None), |(_, turn, target)| (turn, Some(target)))
}

pub(super) fn rotate(
    mesh: &mut SurfaceMeshData,
    first: usize,
    g: &GridGeometry,
    p: &source::Match,
    turn: u8,
) {
    if turn == 0 {
        return;
    }
    let (west, _, north, _) = g.bounds(p.column, p.row);
    let sx = g.tile_width / 8.;
    let sz = g.tile_height / 8.;
    let rotate = |x: f32, z: f32| match turn {
        1 => [z, -x],
        2 => [-x, -z],
        3 => [-z, x],
        _ => unreachable!("quarter turn"),
    };
    // The canonical seat center is(8,4), with its back toward -Z. An east/west
    // chair needs two more source pixels north/south: keep it wholly inside
    // the original16x16 owned plot and four pixels clear of the actor foot.
    let offset = if turn % 2 == 1 { 2. } else { 0. };
    for i in first..mesh.positions.len() {
        let v = mesh.positions[i];
        let [x, z] = rotate((v[0] - west) / sx - 8., (v[2] - north) / sz - 4.);
        mesh.positions[i] = [west + (8. + x) * sx, v[1], north + (4. + z + offset) * sz];
        let old = Vec3::from_array(mesh.normals[i]);
        let [x, z] = rotate(old.x * sx, old.z * sz);
        let normal = Vec3::new(x / sx, old.y, z / sz).normalize();
        mesh.normals[i] = normal.to_array();
        // append_fitted bakes a directional face response. Re-bake that last
        // response after turning, preserving the original material and alpha.
        let shade = crate::interior_models::authored_face_color([1.; 4], normal)[0]
            / crate::interior_models::authored_face_color([1.; 4], old)[0];
        for axis in 0..3 {
            mesh.colors[i][axis] *= shade;
        }
    }
}
